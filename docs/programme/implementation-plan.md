# Initial implementation plan

Status: Working draft

Last reviewed: 27 August 2026

Decision authority: Public Purpose Lab founders

## Purpose

This plan turns the agreed [programme roadmap](roadmap.md) and
[logical architecture](../architecture/logical/README.md) into an ordered set of
delivery milestones for the initial roadmap. It defines dependencies,
deliverables and evidence gates without fixing dates, vendors or an estate of
services prematurely.

This is an enduring delivery plan, not a task backlog. Detailed work items may
change as evidence emerges, but the business outcomes, authority boundaries and
milestone gates remain stable until explicitly reviewed.

Every architecture specification, binding and implementation described here is
the best-known current baseline. Normal iterative development is expected to
refine it. Revisions remain explicit, versioned and evidence-led; the existence
of a baseline neither freezes a mechanism nor permits an enduring security or
authority invariant to change silently.

## Current baseline

The repository currently provides:

- the agreed initial and future programme roadmap;
- the conceptual framework and working logical architecture;
- a verified Rust and TypeScript repository, container and Kubernetes
  skeleton;
- machine-readable catalogues for twenty-one logical components and forty
  contract families;
- separate Workbench, Director and Presentation browser shells;
- accepted `IAM-01` logical responsibilities and `I-001` to `I-005` contract
  semantics, with an accepted local-synthetic M2 reference binding;
- accepted `C-001` to `C-006` schemas and semantics, shared language types and
  an in-development `INT-01` local assurance path;
- an accepted `AUT-01` externalisable authorisation boundary with its
  bounded in-process reference adapter and accepted `AZ-001` contract;
- the accepted M3.1 `CTL-01`, `D-001` to `D-004` and Scenario Director threat
  baseline, with implementation and presentation bindings still pending;
- accepted local-synthetic and managed trust profiles, with managed trust
  required for hosted, shared, production-like or non-synthetic-data use; and
- CI evidence for source checks, tests, builds, Kubernetes rendering and the
  two container images at their stated maturities.

The baseline provides bounded local-synthetic identity and session assurance,
but not external-human identity, managed trust, browser login, an event broker
or external API, workflow, business persistence, retrieval, reporting,
analytics or cREXX execution. The M1 and M2 local journals are limited security
and delivery evidence, not general persistence capabilities. A source package,
working contract or container is not evidence beyond its stated maturity and
conformance profile.

## Delivery principles

1. **Prove one complete business path.** Delivery remains anchored to charity
   systems discovery and reporting, with policy and guidance drift as the
   second initial demonstration.
2. **Stabilise contracts before dependent behaviour.** Documentation and
   implementation may proceed in parallel, but code must not embed an
   unreviewed trust, authority or interoperability decision.
3. **Keep every slice executable.** Each milestone ends in an observable path,
   conformance evidence or a deliberate refusal—not only documents or empty
   packages.
4. **Establish the security model before dependent implementation.** Trust
   zones, principal types, authority flow, information classes, key and secret
   boundaries, recovery domains and failure posture govern the whole framework;
   they are not an IAM feature to retrofit later.
5. **Implement cross-cutting controls from the start.** Minimal identity,
   audit, observability, persistence, failure and recovery behaviour accompanies
   the first runtime interaction even where the complete component
   specification is scheduled later.
6. **Keep logical and physical boundaries distinct.** Logical responsibilities
   may share a process or container until ownership, trust, scaling, resilience
   or independent change provides evidence for separation.
7. **Use synthetic data unless separately authorised.** Public material may be
   used where its rights, provenance and permitted purpose are recorded.
8. **Retain accountable human authority.** Retrieval, AI, rules and workflow
   may propose or support; an authorised person owns interpretation, approval
   and report release.
9. **Qualify local and hosted forms.** The local and Kubernetes-compatible
   editions share contracts, evidence and security expectations even where
   their platform bindings differ.
10. **Make technology replaceable.** Event, storage, identity, AI and retrieval
    products remain behind explicit contracts and require ADRs where the choice
    materially affects the architecture.
11. **Return evidence to the blueprint.** Demonstrator findings, limitations
    and operating evidence are fed back to Architecture Portal without
    overstating adoption or maturity.

## Parallel delivery model

Four coordinated streams run through every milestone:

| Stream                     | Responsibility                                                                                                                       |
| -------------------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| Architecture and contracts | Define component responsibility, transport-neutral contracts, examples, failure behaviour, conformance and required ADRs.            |
| Implementation             | Add only the code, schemas, adapters, storage and interfaces required by the milestone's executable path.                            |
| Demonstration and evidence | Extend scenario fixtures, checkpoints, adverse cases, reports and the evidence pack.                                                 |
| Security and operation     | Provide identity, authority, audit, health, diagnostics, persistence, recovery and supply-chain evidence proportionate to the slice. |

Architecture may run one milestone ahead to reduce idle time. Implementation
may establish a package boundary or test harness early, but behaviour that
depends on a material decision waits for its contract and ADR. A milestone is
complete only when all four streams satisfy its exit gate.

## Foundational security-model gate

The framework security model is the first architecture deliverable of `M1` and
an explicit gate for identity, event, storage and hosted-runtime implementation.
It is maintained as a framework-level architecture document and supported by a
threat model. `IAM-01` applies part of this model but does not own it.

The model must define at least:

- trust zones and boundaries for local, portable and hosted environments;
- the separate external-human, synthetic-human, workload, operator and
  service-owner principal types;
- authentication, authority, delegation, purpose and receiving-component
  decision responsibilities;
- environment identity, workload trust and environment-specific synthetic
  trust domains;
- data and information classes, including credentials, keys, source assets,
  generated analysis, accepted findings, evidence and released reports;
- key generation, secret custody, rotation, revocation, recovery and
  non-exportability expectations by deployment profile;
- visible trust-profile classification and fail-closed compatibility between
  the identity root, hosting/sharing model and information classification;
- separation of identity/trust recovery, security-state recovery and evidence
  or business-data recovery;
- protected command, event, API, browser and component-to-component boundaries;
- least privilege, default refusal and containment of malformed, delayed,
  duplicated or unauthorised work;
- audit, provenance, privacy-minimised observability and support-access
  boundaries;
- dependency, build, image and software-supply-chain trust; and
- the equivalence and known differences between local and hosted enforcement.

The model remains technology-neutral until an ADR selects a binding. It must
show which component owns each decision and which evidence demonstrates the
boundary. No passing build, private network, Kubernetes namespace or access to
an event broker is treated as proof of trust.

The founders completed the `M1` review of the framework security model and its
principal threat cases on 26 August 2026. Acceptance included an externalisable
policy-decision boundary while preserving receiving-component enforcement and
accountability. The accepted M2 local-synthetic conformance evidence shows that
its reference identity and authorisation bindings preserve that model within
their stated single-host boundary. Later milestones extend the threat analysis
when they introduce event delivery, browser control, content, retrieval, AI,
workflow, reporting or external adapters; they do not replace the foundation
silently.

## Milestone sequence

### M0 — Portable framework baseline

Status: Complete

Outcome: A reproducible repository and deployment skeleton exists without
claiming operational platform capabilities.

Delivered:

- Cargo and pnpm workspaces;
- Workbench, Director and Presentation browser shells with a shared UI;
- architecture and contract catalogues;
- framework host and non-operational `IAM-01` package boundary;
- container, Compose and Kubernetes skeletons;
- repository checks and CI; and
- accepted IAM-01 logical semantics.

Evidence gate:

- clean builds and tests pass;
- Kubernetes manifests render;
- both skeleton images build in CI; and
- maturity labels distinguish skeletons from implemented capabilities.

### M1 — Security model, common interaction and runtime spine

Status: Complete — accepted development baseline on 26 August 2026

Outcome: Components can exchange one versioned command, outcome and evidence
reference through an inspectable, testable interaction boundary.

Contract and architecture deliverables:

- define and approve the framework security model and initial threat model;
- specify and approve `C-001` Interaction envelope;
- specify and approve `C-002` Authority and purpose context;
- specify and approve `C-003` Command outcome and failure;
- specify and approve `C-004` Evidence reference;
- specify and approve `C-005` Component capability manifest;
- specify and approve `C-006` Contract compatibility descriptor;
- specify `INT-01` Interaction infrastructure and contract registry; and
- record ADRs for schema representation, compatibility, the first interaction
  binding and any durable idempotency or delivery state.

Implementation deliverables:

- versioned schemas, examples and negative fixtures for the common contracts;
- shared Rust contract types and TypeScript consumption where required;
- a contract catalogue and compatibility check that validates real schemas;
- a minimum command/outcome reference path through the framework host;
- correlation, causation and idempotency handling;
- a minimal audit/evidence append boundary;
- health, readiness and safe diagnostic output; and
- only the persistence needed to prove duplicate, restart and refusal
  behaviour.

Exit evidence:

- one command is accepted or safely refused and produces a correlated outcome;
- repeated delivery cannot duplicate the accepted operation;
- unsupported or incompatible contract versions fail visibly;
- evidence and authority references survive the interaction without carrying
  credentials or sensitive content;
- each interaction crosses only defined trust zones using the appropriate
  principal, authority and information classification;
- logs, traces and failures demonstrate the security model's disclosure and
  support-access boundaries;
- restart behaviour is demonstrated; and
- the same conformance fixtures run against local and container-hosted forms.

Closure evidence:

- the founders accepted the framework security model, M1 threat model, common
  contract semantics and ADRs `0004` to `0006` on 26 August 2026;
- the approved externalisable authorisation boundary is recorded as `AUT-01`,
  without selecting or claiming implementation of a product;
- native checks and builds passed; and
- hosted Linux and container CI passed, including accepted-then-duplicate
  reconciliation across container restart.

M1 closure accepts the architecture and development-assurance baseline. It does
not change `INT-01` from `in-development` or qualify authentication, an external
API, distributed delivery, authoritative audit retention, Windows, high
availability or production security.

### M2 — Environment identity and synthetic access

Status: Complete — accepted local-synthetic development-assurance baseline

Outcome: One environment can establish workload trust and bounded synthetic
application sessions using the accepted IAM semantics.

Contract and architecture deliverables:

- bind `I-001` to `I-005` to the accepted common contracts;
- define schemas, examples and conformance fixtures for all five identity
  families;
- specify `AZ-001` and bind required identity and session actions to the
  accepted `AUT-01` decision/enforcement model;
- record ADRs for the first supported environment identity, protected key,
  signing, workload identity, authorisation engine, authoritative attribute
  sources, session and recovery bindings; and
- bind the accepted `local-synthetic` and `managed` profiles to environment
  classification, readiness, operations and evidence contracts;
- define which deployment profiles support protected same-environment recovery
  and which explicitly rebuild with a new trust domain.

Implementation deliverables:

- environment-specific synthetic trust bootstrap under `I-003`;
- a local-synthetic binding for an isolated scratch profile and an explicit
  fail-closed interface for a future managed-issuer binding;
- workload identity and least-privilege contract authority under `I-002`;
- a bounded `AUT-01` decision path using synthetic relationship and consent
  sources for the demonstration profile;
- synthetic actor registry and role constraints;
- signed, short-lived, single-use grants under `I-004`;
- application-bound establishment, refusal, replay, termination and revocation
  outcomes under `I-005`;
- protected replay, revocation and session state;
- backup/restore or explicit rebuild evidence for each supported profile; and
- `I-001` external-human integration only after its provider and mapping ADR is
  approved; it does not block the synthetic demonstration path.

Local-synthetic exit evidence:

- two local-synthetic environments generate unrelated roots;
- a grant from one environment is refused by the other;
- local-synthetic trust is prominent in operational and evidence views and
  cannot become ready in a hosted, shared, production-like, production or
  non-synthetic-data environment;
- different synthetic human actors can use different registered applications
  in one Demonstration Session;
- each grant and establishment operation creates at most one session despite
  duplicate delivery, restart or lost acknowledgement;
- workload and synthetic-human identities cannot substitute for each other;
- required deny, indeterminate, stale-relationship and unmet-obligation results
  fail closed and cannot be overridden by the receiving component;
- recovery performs the accepted security fix-up or establishes a new trust
  domain; and
- no key, grant, credential or usable session value appears in a URL, event,
  log, trace, analytical record or evidence pack.

Current implementation result:

- canonical `I-001` to `I-005` and `AZ-001` schemas, positive/negative fixtures
  and shared Rust/TypeScript types are in place;
- the Rust host implements local-synthetic environment bootstrap, workload
  authority, bounded policy decisions, signed grants, at-most-one session
  establishment, termination, revocation and rebuild recovery;
- native conformance exercises independent roots, cross-environment refusal,
  concurrent duplicate delivery, restart reconciliation, stale assertion
  refusal, obligation enforcement and journal disclosure checks;
- Compose and Kubernetes examples initialise the local-synthetic profile before
  readiness and expose its warning and limitations; and
- a live Minikube run built the images without Docker Desktop, reached healthy
  identity-required readiness, established a signed synthetic session, proved
  duplicate reconciliation and generated a new trust domain after Pod
  replacement.

The founders accepted the M2 local-synthetic evidence and its stated limits on
26 August 2026. M2 is therefore closed as an in-development reference baseline;
component maturity is not promoted to demonstrated or production-ready.

Managed trust and external-human identity are a separate future binding, not an
unfinished local-synthetic mechanism. Before hosted, shared, production-like,
production or non-synthetic-data use, that work must provide environment-bound
managed signers, authenticated human and workload identities, protected
persistence and recovery, cross-environment refusal and its own threat model,
ADRs and qualification evidence. Until then those declarations remain
fail-closed. `I-001` remains contract-complete but operationally unbound.

### M3 — Scenario Director and presentation control

Status: In progress — M3.1 to M3.3 complete at their stated maturity; M3.4 is
the next delivery slice

Outcome: A repeatable synthetic scenario controls presentation through semantic
events and authenticated surface bindings rather than fragile browser routes.

M3 establishes the demonstration-control plane. It does not implement the
Workbench's asset, retrieval, workflow or reporting capabilities scheduled for
later milestones. Its first executable scenario is deliberately small: one
Director, at least two registered surface roles, named synthetic actors and a
sequence of semantic cues, checkpoints and safe adverse cases.

Logical responsibilities:

| Responsibility                                                                                  | Owner                                             | M3 boundary                                                                                        |
| ----------------------------------------------------------------------------------------------- | ------------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| Scenario package, Demonstration Session, lifecycle, controlled time, reset plan and checkpoints | `CTL-01` Scenario Director                        | Coordinates components but never owns their business data, identity decisions or approvals.        |
| Surface capability, registration, binding, cue delivery and delivery outcome                    | `CTL-02` Presentation Gateway and screen registry | Knows semantic surface capabilities and safe bindings, not internal browser routes or credentials. |
| Presenter controls, readiness, progress, refusal and recovery views                             | `UX-03` Director Console                          | A user interface over authorised `CTL-01` actions; it is not the authoritative scenario state.     |
| Semantic view resolution and audience-facing presentation                                       | `UX-04` Presentation Surface                      | May change the current view; it cannot mutate business state or validate a sign-in grant.          |
| Message semantics, correlation, compatibility and bounded delivery evidence                     | `INT-01`                                          | Carries versioned contracts; transport selection remains a replaceable binding.                    |
| Presenter, workload, synthetic actor and application-session authority                          | `IAM-01` and `AUT-01`                             | Preserves the three identity paths and fail-closed receiving-component enforcement accepted in M2. |

The initial physical composition may place `CTL-01` and `CTL-02` in one Rust
deployable and share existing frontend packages. Their state, contracts and
authority remain logically separate so evidence can justify a later split.

Contract and architecture deliverables:

- specify `CTL-01` ownership, state, lifecycle, failure and recovery and approve
  `D-001` Scenario Package, `D-002` Scenario Lifecycle, `D-003` Reset, Clock and
  Fault Control and `D-004` Readiness and Checkpoint;
- specify `CTL-02` ownership, state, binding, delivery and recovery and approve
  `P-001` Presentation Capability Manifest, `P-002` Presentation Surface
  Registration, `P-003` Presentation Cue and `P-004` Presentation Cue Outcome;
- specify the relevant `UX-01`, `UX-03` and `UX-04` responsibilities,
  including accessibility, reconnect, session visibility and safe failure;
- extend the M2 threat model for presenter authority, surface impersonation,
  cue injection, replay, route disclosure, cross-session delivery and
  presentation/business-state confusion; and
- record ADRs for scenario-package integrity, the first event transport and
  delivery profile, presenter authentication, surface binding, controlled
  time, reset/fault authority and restart recovery.

Required interaction sequence:

1. `CTL-01` validates a versioned scenario package and its required component,
   actor, surface and evidence capabilities.
2. `CTL-02` authenticates each surface workload or session, records its
   `P-001` capabilities and binds it through `P-002` to one Demonstration
   Session and screen role.
3. `CTL-01` requests any required synthetic sign-in through the M2 identity
   path; applications establish ordinary synthetic sessions without exposing
   grants to browser content or URLs.
4. An authorised presenter issues a `D-002` lifecycle action. `CTL-01` records
   the outcome and emits separately authorised business commands where the
   scenario requires business change.
5. `CTL-01` emits a short-lived `P-003` semantic cue. `CTL-02` routes it to the
   named registered surface, which resolves the semantic view locally and
   returns `P-004`.
6. `CTL-01` evaluates `D-004` checkpoints from observed facts. Presentation
   progress, software readiness and business completion remain separate.

Implementation deliverables:

- one versioned assurance scenario package with synthetic actor, surface,
  lifecycle, cue, checkpoint and reset fixtures;
- durable single-instance start, pause, resume, stop, reset and checkpoint
  handling with idempotent commands and explicit outcomes;
- one replaceable event-transport binding that uses the accepted common
  envelopes and can run locally and in Kubernetes without treating Kubernetes
  Events as the application message bus;
- authenticated, expiring surface registration and capability discovery with
  reconnect and stale-binding handling;
- short-lived semantic presentation cues and applied, refused, unsupported,
  expired and failed outcomes;
- Director views for readiness, progress, refusal and failure;
- Presentation views that resolve cues without receiving business authority;
- multi-actor synthetic sign-in across the scenario's registered surfaces;
- bounded operational status and an evidence record for lifecycle, cue and
  checkpoint outcomes without credentials or usable session values; and
- local-container and Minikube deployment examples using the common M1/M2
  components and security posture; and
- the ADR-0012 Google Cloud preview lifecycle: one disposable infrastructure
  create/destroy spike, followed only after the local walking skeleton by a
  private, time-bounded hosted smoke with explicit trust and cost evidence.

Delivery sequence:

1. **M3.1 — contracts and threats — accepted 27 August 2026:** `CTL-01`,
   `D-001` to `D-004`, ADR-0011 and the M3 logical threat-model baseline are
   accepted. This closes the logical design slice only; schemas, implementation
   and physical bindings remain pending.
2. **M3.2 — presentation and hosted-lifecycle binding — design accepted and
   infrastructure spike completed 27 August 2026:** `CTL-02`, `P-001` to
   `P-004`, ADR-0013 to ADR-0016 and the M3.2 threat extension are accepted.
   The disposable infrastructure create/manual-off result is recorded in the
   [M3.2 hosted-lifecycle evidence](../architecture/evidence/m3-2-google-cloud-hosted-lifecycle-spike.md);
   no hosted application claim follows from it.
3. **M3.3 — runtime walking skeleton — completed 28 August 2026:** one
   Director/runtime, registry and Presentation Surface path exercises lifecycle,
   logical time, bounded cue fault, semantic cue/outcome, checkpoint, stop and
   successor reset natively and in independent OCI Compose and Minikube
   profiles. The reviewed source was published as an immutable image; a private,
   ingress-free Google Cloud activation passed liveness, contract/package and
   fail-closed checks before explicit teardown. Cost actuals remain a delayed
   operational read-back, recorded as pending rather than zero.
4. **M3.4 — identity and resilience:** integrate M2 synthetic sign-in,
   reconnect, duplicate, expiry, restart, reset and safe-failure cases. Before
   any shared hosted demonstration, add the narrow managed-root, presenter,
   workload, protected-state and authorised-activation binding required by
   ADR-0007 and ADR-0012.
5. **M3.5 — demonstration evidence:** run the repeatable local and Minikube
   scenario and one scheduled, automatically expiring hosted preview; record
   limitations, gross usage, credits, net cost and teardown evidence for
   founder review.

The accepted
[M3.3 runtime walking-skeleton baseline](../architecture/m3-3-runtime-walking-skeleton-baseline.md)
and ADR-0017 to ADR-0020 define the current package, component-state,
controlled-time/reset/fault and deployable bindings. Founder acceptance permits
implementation against this baseline. The repository now contains that
implementation plus native, independent OCI Compose, Minikube and exact-image
private Google Cloud evidence. The accepted gates are conclusive for the
synthetic-only, in-development M3.3 claim; M3.4 remains the next authority and
security gate.

M3.1 is closed as an accepted logical baseline. M3.2 accepts NATS JetStream,
Google OIDC, the backend-mediated SSE/POST surface channel and the ephemeral GKE
Autopilot/OpenTofu hosted lifecycle at design maturity. M3.3 accepts canonical
JSON packages, component-owned SQLite stores, separate operational/logical
time, bounded reset/fault adapters and one application image with separate
Director and Presentation Gateway workloads. Exact library versions and
numerical limits are recorded by the implementation and remain a revisable
current baseline rather than a production qualification.

The M3.2 infrastructure sub-slice is evidenced: short-lived operator
federation, pre-apply expiry arming, private-node Autopilot create, explicit
off, partial-failure recovery and empty residual activation inventory were
observed. M3.3 additionally evidenced exact-image application deployment,
expected fail-closed hosted behaviour and conclusive teardown. Managed trust,
presenter/workload binding and synthetic application sign-in remain M3.4 work;
automatic-expiry execution remains an M3.5 evidence case.

### Hosted-preview scheduling and cost boundary

ADR-0012 brings cloud lifecycle evidence forward without moving formal hosted
qualification out of M6. The local environment remains the normal development
path. The hosted preview defaults to `off`; `on` is restricted to named
operators, records an automatic expiry and must return a conclusive teardown
outcome. An optional continuing `warm-off` state requires measured justification.

The provisional operating target is an average net cost of approximately 30
units in the billing account's currency per month. Evidence reports gross usage,
credits and net cost separately so a temporary credit does not hide the
underlying design cost. Exact billing configuration and balances remain in
protected infrastructure records.

The public static website is not the demonstrator control plane and does not
keep the hosted environment active. ADR-0016 accepts the first GKE Autopilot,
Workload Identity Federation, Cloud KMS, Cloud Tasks/Cloud Build and OpenTofu
binding. Exact account configuration and later application persistence remain
protected or deferred implementation decisions.

Exit evidence:

- the scenario can be repeated from a known reset state;
- only an authenticated and authorised presenter/workload can control a
  Demonstration Session, and a surface cannot register into another session;
- Director restart, gateway restart, screen refresh and reconnect do not
  corrupt business or synthetic-session state;
- delayed, duplicate, expired and unsupported cues produce safe outcomes;
- a cue is resolved by semantic capability and no cue, event, evidence record
  or browser history carries a route, grant, credential or usable session
  value;
- presentation cues cannot produce hidden business-state mutation and business
  commands remain separately authorised and enforced by their owners;
- reset, controlled time and fault actions affect only their approved scope and
  are visible in evidence;
- checkpoints distinguish presentation progress, software health and business
  completion; and
- the same package supports live demonstration and automated adverse-case
  evidence in local-container and Minikube profiles; and
- the private Google Cloud preview can be created, activated by an authorised
  operator, automatically expired and deactivated without accepting a
  local-synthetic root or leaving an unintended runtime endpoint, workload or
  unexplained material cost.

M3 does not deliver Workbench asset handling, RAG, workflow, domain reporting,
general managed external-human identity, real data, multi-tenant operation,
high availability or production browser-session security. The hosted preview's
narrow managed root and operator/presenter binding do not qualify those broader
capabilities. They remain in later milestones or require separately qualified
bindings.

### M4 — Governed source and evidence workbench

Status: Planned

Outcome: A practitioner can establish a synthetic engagement, register or
acquire sources, stage an approved version and obtain a bounded cited evidence
packet.

Contract and architecture deliverables:

- specify `DOM-01` and approve `E-001` for the minimum engagement record;
- specify `CNT-01` and approve `A-001` and `A-002`;
- specify `KNO-01` and approve `K-001` and `K-002`;
- specify the required `UX-02` Workbench responsibilities;
- specify the bounded `AIO-01` and `ADP-01` responsibilities used by this slice;
  and
- record ADRs for owned metadata, content storage, quarantine, retrieval
  binding and AI/provider use.

Implementation deliverables:

- bounded engagement, purpose and authority records;
- source link/upload, acquisition, validation, classification, rights,
  immutable version and staging state;
- quarantine and explicit reviewer release into knowledge processing;
- replaceable knowledge-ingestion and query interfaces;
- qualification of `crexx-rag` as the preferred first retrieval component,
  without making it the asset register, system of record or policy authority;
- cited passages, claims, relationships, conflicts, gaps and uncertainty; and
- Workbench views for assets, staging, query and evidence review.

Exit evidence:

- the lifecycle `register → acquire → validate → version → stage → retrieve →
review` is demonstrated;
- malformed, unauthorised or unsupported material is contained and explained;
- retrieval is limited to approved staged versions;
- every returned claim points to retained source evidence and processing
  provenance;
- conflicting or missing evidence remains visible; and
- provider identity, model/configuration, resource use, failure and abstention
  are recorded where AI is used.

### M5 — Accountable work, rules and reports

Status: Planned

Outcome: Proposed findings move through accountable human work into a versioned,
evidence-linked report element and guidance-drift report.

Contract and architecture deliverables:

- specify `WRK-01` and approve `W-001`;
- specify `RUL-01` and approve `R-001`;
- specify `RPT-01` and approve `RP-001` to `RP-004`;
- specify `AUD-01` and `ANA-01` and approve `AU-001`, `AN-001` and `AN-002`;
- complete the `AI-001` and `X-001` bindings required by the slice; and
- record any cREXX runtime, workflow, report rendering, diagram or analytical
  representation decisions as ADRs.

Implementation deliverables:

- durable work ownership, review, approval, rejection, escalation and history;
- explicit separation of generated claims, accepted findings and released
  outputs;
- bounded versioned rule or transformation execution with explanations;
- cREXX integration where its inspected value justifies it, without delaying a
  simpler first rule path;
- report preview, accountable release, correction and retraction states;
- relationship graph, analytical view and declarative diagram artifacts;
- append-oriented audit reconstruction and reproducible projections; and
- charity discovery and policy-drift demonstration reports.

Exit evidence:

- a source-backed proposed finding cannot become accepted without authorised
  human action;
- retrieved prose cannot silently become active policy or operational logic;
- every report element traces to sources, processing, rules and decisions;
- repeated, refused, failed and partially completed work remains recoverable;
- report limitations and authority are visible; and
- the Demonstration Evidence Pack reconstructs the complete path.

### M6 — Initial-roadmap operational qualification

Status: Planned

Outcome: The first business path is repeatable in supported local and hosted
profiles with evidence proportionate to an initial demonstrator.

Contract and architecture deliverables:

- complete `OPS-01` and `PLT-01` with `O-001`, `O-002`, `L-001` and `L-002`;
- complete any deferred deployment, persistence, backup, restore, reset,
  observability and supply-chain ADRs; and
- reconcile the implemented profile with the Architecture Portal blueprint.

Implementation deliverables:

- supported local installation and Kubernetes-compatible hosted composition;
- owned database and content-storage boundaries without one integration-wide
  shared database;
- health, readiness, logs, metrics, traces and safe support views;
- failed-work inspection and controlled recovery;
- environment reset, backup and restore, including IAM recovery-domain
  separation;
- controlled fault and adverse-input scenarios;
- build provenance, dependency and image evidence; and
- recorded value, effort, provider use, operating cost and limitations.

Exit evidence:

- the roadmap's initial completion evidence is satisfied for one bounded
  synthetic engagement in local and hosted modes;
- software health remains distinguishable from business completion;
- recovery and reset do not duplicate accepted work or silently activate
  uncertain state;
- privacy, identity, evidence and failure boundaries remain observable; and
- no result is represented as legal, regulatory, clinical or production
  assurance.

## Cross-milestone evidence register

Every milestone records:

- accepted component and contract versions;
- ADRs and unresolved decisions;
- executable examples and negative fixtures;
- test, build and supported-platform evidence;
- scenario checkpoints and failure/recovery outcomes;
- security, privacy and authority limitations;
- provider, dependency and supply-chain provenance;
- value, effort, resource and operating-cost observations; and
- lessons returned to the roadmap and Architecture Portal blueprint.

Evidence from one operating system, deployment profile or scenario does not
qualify another unless the relevant boundary is demonstrably identical.

## Initial-roadmap exclusions

This plan does not authorise:

- real personal or client-confidential data;
- live charity, NHS, government or social-care integration;
- legal, regulatory, safeguarding, clinical or professional certification;
- autonomous consequential decisions or policy activation;
- production multi-tenancy or a production service commitment;
- use of personal subscription credentials as shared hosted credentials;
- a service or Kubernetes workload for every logical component; or
- publication of an unqualified component as stable or production-ready.

Those capabilities remain in the future roadmap or require separately recorded
authority, governance and assurance.

## Change and approval

A milestone may start when its purpose and prerequisite contracts are clear.
Material architecture, security, privacy, licensing, interoperability and
deployment decisions require ADRs. A milestone moves to complete only after its
exit evidence is reviewed; source completion or a passing build alone is not
sufficient.

The founders approve changes to milestone outcomes or programme boundaries.
Component and contract details follow the acceptance process in the logical
architecture. Implementation maturity is recorded separately from document
approval so that an accepted design cannot be mistaken for a delivered
capability.

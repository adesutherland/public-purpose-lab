# Initial implementation plan

Status: Working draft

Last reviewed: 26 August 2026

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
  semantics;
- accepted `C-001` to `C-006` schemas and semantics, shared language types and
  an in-development `INT-01` local assurance path;
- an accepted `AUT-01` externalisable authorisation boundary with its
  implementation and `AZ-001` contract deliberately planned for later work;
  and
- CI evidence for source checks, tests, builds, Kubernetes rendering and the
  two container images at their stated maturities.

The baseline does not provide operational identity, synthetic sessions, an
event broker or external API, workflow, business persistence, retrieval,
reporting, analytics or cREXX execution. The M1 local journal is limited
delivery and audit evidence, not a general persistence capability. A source
package, working contract or container is not evidence beyond its stated
maturity and conformance profile.

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
accountability. Before `M2` identity and authorisation behaviour is accepted,
conformance evidence must show that the selected bindings preserve that model.
Later milestones extend the threat analysis when they introduce content,
retrieval, AI, workflow, reporting or external adapters; they do not replace
the foundation silently.

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

Status: Planned

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
- define which deployment profiles support protected same-environment recovery
  and which explicitly rebuild with a new trust domain.

Implementation deliverables:

- environment-specific synthetic trust bootstrap under `I-003`;
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

Exit evidence:

- two environments generate unrelated synthetic roots;
- a grant from one environment is refused by the other;
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

### M3 — Scenario Director and presentation control

Status: Planned

Outcome: A repeatable synthetic scenario controls presentation through semantic
events and authenticated surface bindings rather than fragile browser routes.

Contract and architecture deliverables:

- specify `CTL-01` and approve `D-001` to `D-004`;
- specify `CTL-02` and approve `P-001` to `P-004`;
- complete the relevant `UX-01`, `UX-03` and `UX-04` responsibilities; and
- record ADRs for scenario packages, controlled time, reset, surface binding
  and the first presentation-event binding.

Implementation deliverables:

- versioned scenario packages and synthetic actor fixtures;
- start, pause, resume, stop, reset and checkpoint handling;
- authenticated surface registration and capability discovery;
- short-lived semantic presentation cues and explicit outcomes;
- Director views for readiness, progress, refusal and failure;
- Presentation views that resolve cues without receiving business authority;
  and
- multi-actor synthetic sign-in across the scenario's registered surfaces.

Exit evidence:

- the scenario can be repeated from a known reset state;
- Director restart or screen refresh does not corrupt business or session
  state;
- delayed, duplicate, expired and unsupported cues produce safe outcomes;
- no cue carries a route, credential or hidden business-state mutation; and
- checkpoints distinguish presentation progress, software health and business
  completion.

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

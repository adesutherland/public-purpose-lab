# Logical architecture overview

Status: Working draft

Last reviewed: 25 August 2026

## Purpose

This document starts the logical architecture for Public Purpose Lab. It turns
the [agreed conceptual framework](../framework-conceptual-overview.md) into a
controlled catalogue of components and contract families before physical
services, products and deployment topology are selected.

It is an overview and work sequence, not yet the complete blueprint. Each
component and contract will receive a separate, reviewed specification. The
catalogue makes that work finite and traceable while allowing evidence from the
first demonstrator to refine the boundaries.

## Logical-architecture rules

- A component is a responsibility with an owner, state boundary and failure
  behaviour. It is not automatically a service, container or database.
- A contract defines meaning, authority, inputs, outcomes, failure and
  compatibility. A broker topic, HTTP route or library call is only a transport
  binding.
- Commands request an authorised change; events report accepted facts; queries
  retrieve information without changing ownership; presentation cues affect a
  screen but not business state.
- Components integrate through contracts, never by reading another component's
  private tables.
- Human, synthetic and workload identities remain distinct. Synthetic trust is
  generated and valid within one environment only.
- Shared access-control policy is evaluated through `AUT-01`; receiving
  components remain enforcement points and accountable owners of protected
  actions.
- Local and hosted deployments preserve the same logical ownership and contract
  semantics even when responsibilities are physically combined.
- Every detailed specification must describe refusal, repeated delivery,
  partial failure, recovery, audit and conformance evidence.

The identifiers below are stable working references for the detailed design.
Renaming or splitting one requires the overview and affected contracts to be
updated together.

## Logical component catalogue

### Experience and presentation

| ID      | Component                  | Owned responsibility                                                                                           | Principal contract families                                        |
| ------- | -------------------------- | -------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| `UX-01` | Common frontend platform   | Accessible design system, shell, navigation, session integration and shared interaction patterns.              | `C-005`, `I-001`, `I-005`, `O-001`                                 |
| `UX-02` | Service Evidence Workbench | Practitioner and client interaction for engagements, assets, evidence, work, visualisation and report release. | `E-001`, `A-001`, `K-002`, `W-001`, `RP-001` to `RP-004`, `AN-002` |
| `UX-03` | Director Console           | Presenter controls and views for sessions, screens, checkpoints, readiness and failures.                       | `D-001` to `D-004`, `P-002` to `P-004`, `I-004`, `O-001`           |
| `UX-04` | Presentation Surface       | A registered screen that resolves semantic views and applies authorised presentation cues.                     | `P-001` to `P-004`, `I-005`, `O-001`                               |

### Control, identity and interaction

| ID                                                                                | Component                                        | Owned responsibility                                                                                                             | Principal contract families                                           |
| --------------------------------------------------------------------------------- | ------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------- |
| `CTL-01`                                                                          | Scenario Director                                | Scenario definitions, demonstration execution, checkpoints, controlled time, faults and evidence assembly.                       | `D-001` to `D-004`, `P-003`, `I-004`, `AU-001`                        |
| `CTL-02`                                                                          | Presentation Gateway and screen registry         | Surface registration, capability discovery, cue routing, expiry and acknowledgements.                                            | `P-001` to `P-004`, `I-002`, `I-005`, `O-001`                         |
| [`IAM-01`](components/iam-01-identity-trust-and-synthetic-session-broker.md)      | Identity, trust and synthetic session broker     | External identity context, workload trust, visible local-synthetic or managed trust profile, bounded grants, roles and sessions. | `I-001` to `I-005`, `C-002`, `AU-001`                                 |
| [`AUT-01`](components/aut-01-policy-decision-and-authorisation.md)                | Policy decision and authorisation                | Versioned shared access-control decisions over bounded identity, purpose, resource, relationship and environmental attributes.   | `AZ-001`, `C-002` to `C-004`, `AU-001`, `O-001`                       |
| [`INT-01`](components/int-01-interaction-infrastructure-and-contract-registry.md) | Interaction infrastructure and contract registry | Message carriage, schema and contract publication, compatibility evidence, delivery state and consumer coordination.             | `C-001` to `C-006`, plus transport bindings for every public contract |

### Engagement, content, knowledge and work

| ID       | Component                            | Owned responsibility                                                                                                    | Principal contract families                      |
| -------- | ------------------------------------ | ----------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------ |
| `DOM-01` | Engagement and domain records        | Engagement purpose, authority, scope and authoritative organisational model.                                            | `E-001`, `C-002`, `C-004`, `AU-001`              |
| `CNT-01` | Assets, content and source staging   | Asset register, acquisition, quarantine, immutable versions, provenance, classification, rights and release to staging. | `A-001`, `A-002`, `K-001`, `C-004`, `AU-001`     |
| `KNO-01` | Knowledge, retrieval and evidence    | Source passages, claims, relationships, ambiguity, conflicts, gaps and cited retrieval.                                 | `K-001`, `K-002`, `C-004`, `AU-001`              |
| `WRK-01` | Work, case and workflow              | Work identity, queues, responsibility, deadlines, escalation, completion and history.                                   | `W-001`, `I-001`, `R-001`, `AU-001`, `AN-001`    |
| `RUL-01` | Rules, decisions and transformations | Bounded, versioned execution of explicit rules and transformations with explanations.                                   | `R-001`, `C-002`, `C-004`, `AU-001`              |
| `AIO-01` | Bounded AI and tool orchestration    | One controlled computational run across models, retrieval, tools and human-input steps.                                 | `AI-001`, `K-002`, `W-001`, `C-004`, `AU-001`    |
| `RPT-01` | Reports, visualisations and diagrams | Definitions, previews and artifacts for reports, relationship graphs, analytical views and declarative diagrams.        | `RP-001` to `RP-004`, `K-002`, `AN-002`, `C-004` |
| `ADP-01` | Integration and adapters             | Translation and isolation of authorised external or simulated interfaces.                                               | `X-001`, `C-001` to `C-004`, `AU-001`            |

### Evidence, learning and operation

| ID       | Component                             | Owned responsibility                                                                                                | Principal contract families                                         |
| -------- | ------------------------------------- | ------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------- |
| `AUD-01` | Audit and provenance                  | Append-oriented attribution, lineage, decision, access and release evidence.                                        | `AU-001`, `C-004` and evidence-bearing outcomes from all components |
| `ANA-01` | Analytics and projections             | Versioned measures, reproducible projections and analytical query results.                                          | `AN-001`, `AN-002`, accepted events through `C-001`                 |
| `OPS-01` | Observability, operations and support | Health, logs, metrics, traces, alerts, failed-work visibility, safe trust-profile status and recovery coordination. | `O-001`, `O-002`, `C-003`                                           |
| `PLT-01` | Platform and delivery                 | Builds, supply chain, environment bootstrap, configuration, secrets, persistence, backup, restore and deployment.   | `L-001`, `L-002`, `I-003`, `O-001`, `O-002`                         |

The current catalogue contains twenty-one logical components. An early local or
hosted composition may implement several in one deployable unit, but it must
not collapse their ownership or authority boundaries.

## Contract-family catalogue

A contract family groups closely related command, event, query and outcome
variants. A detailed specification may split a family into separate schemas
when different authority, delivery or compatibility rules require it.

### Common interaction contracts

| ID                                                                     | Working name                      | Main participants                             | Enduring purpose                                                                                                                                   |
| ---------------------------------------------------------------------- | --------------------------------- | --------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| [`C-001`](contracts/common/c-001-interaction-envelope.md)              | Interaction envelope              | All public-contract participants              | Carries identity, type, version, issuer, audience, time, correlation, causation, idempotency, purpose, classification and security metadata.       |
| [`C-002`](contracts/common/c-002-authority-and-purpose-context.md)     | Authority and purpose context     | `IAM-01`, `AUT-01` and every command receiver | States actor, role, delegated authority, environment, purpose and constraints used to authorise a request.                                         |
| [`C-003`](contracts/common/c-003-command-outcome-and-failure.md)       | Command outcome and failure       | Every command receiver and caller             | Reports acceptance, refusal, expiry, duplicate handling, failure and recovery ownership without inventing a business fact.                         |
| [`C-004`](contracts/common/c-004-evidence-reference.md)                | Evidence reference                | Evidence-producing and consuming components   | Links a claim, action, rule, source, transformation, model step or release to retained evidence and provenance.                                    |
| [`C-005`](contracts/common/c-005-component-capability-manifest.md)     | Component capability manifest     | User interfaces, gateways and components      | Publishes stable semantic capabilities, contract versions, readiness dependencies and supported profiles without exposing internal implementation. |
| [`C-006`](contracts/common/c-006-contract-compatibility-descriptor.md) | Contract compatibility descriptor | `INT-01`, producers and consumers             | Defines schema status, compatibility, deprecation, examples and conformance evidence.                                                              |

### Demonstration and presentation contracts

| ID      | Working name                      | Main participants                                 | Enduring purpose                                                                                            |
| ------- | --------------------------------- | ------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| `D-001` | Scenario package                  | `CTL-01`, `PLT-01`, scenario components           | Defines synthetic fixtures, actors, stages, commands, cues, checkpoints, reset scope and expected evidence. |
| `D-002` | Scenario lifecycle                | `UX-03`, `CTL-01`, participating components       | Starts, pauses, resumes, stops and reports one Demonstration Session.                                       |
| `D-003` | Reset, clock and fault control    | `CTL-01`, authorised test adapters and components | Applies bounded reset, synthetic time and approved failure injection without bypassing component ownership. |
| `D-004` | Readiness and checkpoint          | `CTL-01`, `OPS-01`, participating components      | Reports prerequisites, observed progress and verifiable checkpoint outcomes.                                |
| `P-001` | Presentation capability manifest  | `UX-04`, `CTL-02`                                 | Declares semantic views, accepted context, version and presentation constraints.                            |
| `P-002` | Presentation surface registration | `UX-04`, `CTL-02`, `IAM-01`                       | Binds an authenticated screen and its capabilities to one Demonstration Session and screen role.            |
| `P-003` | Presentation cue                  | `CTL-01`, `CTL-02`, `UX-04`                       | Requests a short-lived semantic view without carrying a route or changing business state.                   |
| `P-004` | Presentation cue outcome          | `UX-04`, `CTL-02`, `CTL-01`                       | Reports applied, refused, unsupported, expired or failed cue handling.                                      |

### Identity and trust contracts

| ID                                                                      | Working name                     | Main participants                                           | Enduring purpose                                                                                                                  |
| ----------------------------------------------------------------------- | -------------------------------- | ----------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| [`I-001`](contracts/identity/i-001-external-human-identity-context.md)  | External human identity context  | External identity provider, `IAM-01`, authorised components | Supplies validated human identity, role and authority without exposing external credentials.                                      |
| [`I-002`](contracts/identity/i-002-workload-identity-context.md)        | Workload identity context        | `PLT-01`, `IAM-01`, service components                      | Authenticates components and carries least-privilege workload authority.                                                          |
| [`I-003`](contracts/identity/i-003-synthetic-trust-bootstrap-record.md) | Synthetic trust bootstrap record | `PLT-01`, `IAM-01`, `AUD-01`                                | Records environment identity, root creation, trusted signers, rotation state and recovery boundary without exposing private keys. |
| [`I-004`](contracts/identity/i-004-demonstration-sign-in-grant.md)      | Demonstration sign-in grant      | `CTL-01`, demonstration signer, `IAM-01`                    | Requests and issues a signed, short-lived, one-time, environment-bound grant for a named synthetic actor and surface.             |
| [`I-005`](contracts/identity/i-005-synthetic-session-outcome.md)        | Synthetic session outcome        | `IAM-01`, `CTL-02`, target application, `CTL-01`            | Reports session establishment, refusal, expiry, replay, revocation and termination without exposing a usable credential.          |

The linked `C-001` to `C-006` and `INT-01` specifications are accepted M1
semantics. Their schemas and local reference implementation provide the
accepted M1 development-assurance baseline; `INT-01` remains in development and
is not production-qualified.

The linked `IAM-01` and `I-001` to `I-005` logical specifications are accepted.
Their local-synthetic reference implementation provides the accepted M2
development-assurance baseline; `IAM-01` remains in development and is not a
managed, external-human or production identity capability.

### Authorisation contract

| ID                                                                                   | Working name                           | Main participants                                               | Enduring purpose                                                                                                                                      |
| ------------------------------------------------------------------------------------ | -------------------------------------- | --------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| [`AZ-001`](contracts/authorisation/az-001-authorisation-decision-and-obligations.md) | Authorisation decision and obligations | `AUT-01`, `IAM-01`, authoritative sources, receiving components | Evaluates versioned access-control policy and returns permit, deny, not-applicable or indeterminate with obligations and bounded evidence references. |

`AUT-01` and `AZ-001` are accepted logical boundaries. ADR-0009 selects a
bounded in-process M2 reference adapter while leaving the future policy engine,
policy language, deployment topology and authoritative relationship source
replaceable.

### Service and evidence contracts

| ID       | Working name                             | Main participants                                            | Enduring purpose                                                                                                       |
| -------- | ---------------------------------------- | ------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------- |
| `E-001`  | Engagement record command and fact       | `UX-02`, `DOM-01`, authorised service components             | Creates and changes bounded engagement purpose, scope, participation and organisational facts.                         |
| `A-001`  | Asset registration and acquisition       | `UX-02`, `CNT-01`, `ADP-01`                                  | Registers a link or upload and reports acquisition, classification, quarantine, validation or refusal.                 |
| `A-002`  | Staged source release                    | `CNT-01`, authorised reviewer, `KNO-01`                      | Releases one immutable, validated source version for bounded knowledge processing.                                     |
| `K-001`  | Knowledge ingestion                      | `CNT-01`, `KNO-01`                                           | Requests and reports ingestion of a staged source while preserving provenance, conflict and processing status.         |
| `K-002`  | Bounded evidence query and packet        | `UX-02` or `AIO-01`, `KNO-01`, `RPT-01`                      | Returns cited passages, claims, relationships, ambiguity, gaps and limits for one bounded question.                    |
| `W-001`  | Work lifecycle                           | `UX-02`, domain components, `WRK-01`, `RUL-01`               | Creates, allocates, claims, releases, escalates, completes or refuses durable work and reports accepted state changes. |
| `R-001`  | Rule invocation and result               | Authorised components, `RUL-01`, `AUD-01`                    | Supplies defined facts and rule version and returns a bounded outcome, explanation and execution evidence.             |
| `AI-001` | AI and tool execution                    | `UX-02` or service component, `AIO-01`, providers and tools  | Runs one bounded job and reports models, tools, inputs, outputs, limits, resource use, abstention and failure.         |
| `RP-001` | Report composition and release           | `UX-02`, `RPT-01`, authorised releaser                       | Creates, previews, approves, releases, corrects or retracts a versioned evidence-linked report.                        |
| `RP-002` | Relationship graph definition and result | `UX-02`, `RPT-01`, `DOM-01`, `KNO-01`                        | Defines and returns inspectable nodes, edges, lineage, evidence and view constraints.                                  |
| `RP-003` | Analytical view request and artifact     | `UX-02`, `RPT-01`, `ANA-01`                                  | Requests and renders a governed chart or table from a versioned analytical result.                                     |
| `RP-004` | Declarative diagram render               | `UX-02`, `RPT-01`                                            | Renders versioned diagram source and records renderer, inputs, output, evidence and release state.                     |
| `X-001`  | Adapter action and delivery outcome      | Authorised component, `ADP-01`, simulated or external system | Translates an authorised action and reports external acceptance, refusal, delivery, partial completion or failure.     |

### Audit, analytics and operating contracts

| ID       | Working name                         | Main participants                                                 | Enduring purpose                                                                                                                  |
| -------- | ------------------------------------ | ----------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| `AU-001` | Audit and provenance append and read | All evidence-producing components, `AUD-01`, authorised reviewers | Retains attributable, append-oriented evidence and supplies governed reconstruction views.                                        |
| `AN-001` | Projection definition and build      | `ANA-01`, event producers, authorised analysts                    | Versions analytical inputs and calculations and builds reproducible projections from governed facts.                              |
| `AN-002` | Analytical query and result          | `UX-02`, `RPT-01`, `ANA-01`                                       | Returns a versioned measure set with calculation, time basis, provenance and limitations.                                         |
| `O-001`  | Health and readiness                 | Components, `OPS-01`, `CTL-01`, `PLT-01`                          | Reports software health, dependencies, active trust profile and readiness separately from business completion.                    |
| `O-002`  | Fault and recovery                   | Components, `OPS-01`, `PLT-01`, authorised operator               | Records failure, containment, recovery ownership, action and verified restoration.                                                |
| `L-001`  | Component delivery manifest          | `PLT-01`, component owners, `OPS-01`                              | Identifies package, provenance, interfaces, configuration, dependencies, health checks and deployment evidence.                   |
| `L-002`  | Environment profile and bootstrap    | `PLT-01`, `IAM-01`, operators                                     | Defines local, portable, hosted or assurance profile configuration, synthetic-root creation, persistence and recovery boundaries. |

The catalogue currently contains forty contract families. The first
detailed pass will confirm whether each remains one family or separates into
multiple commands, events, queries and outcomes.

## Shared contract template

Every detailed contract specification will state:

1. purpose, status, owner and semantic type;
2. producer, consumer, actor, authority and trust boundary;
3. preconditions, payload concepts and information classification;
4. common-envelope fields and contract-specific fields;
5. acceptance, refusal, expiry, duplication and failure outcomes;
6. correlation, causation, idempotency and ordering rules;
7. retention, provenance, audit and analytical use;
8. schema versioning, compatibility and deprecation;
9. transport-neutral examples, including negative cases; and
10. conformance tests and evidence needed for acceptance.

Sensitive credential or key material is never part of an example, log, event
payload or retained evidence packet.

## Shared component template

Every detailed component specification will state:

1. responsibility, non-responsibilities and accountable owner;
2. users, operators and human decision rights;
3. commands, queries and configuration it accepts;
4. facts, views, artifacts and evidence it produces;
5. information it owns, references, versions and retains;
6. identity, authority, privacy and security boundaries;
7. repeated, delayed, refused and partial-work behaviour;
8. failure containment, restart, recovery and support ownership;
9. audit, observability and analytical obligations;
10. local, portable and hosted deployment considerations;
11. implementation fit for Rust, cREXX, TypeScript or another mechanism;
12. dependencies, replaceability and portfolio relationship; and
13. tests, limitations, maturity and decisions still requiring an ADR.

## First walking-skeleton path

The initial component and contract detail will remain anchored to one complete
charity discovery and reporting path:

> `CTL-01` Scenario Director → `CNT-01` source acquisition and staging →
> `KNO-01` cited retrieval → `WRK-01` human review → `RUL-01` bounded
> transformation → `RPT-01` report element → `AUD-01` evidence and `ANA-01`
> projection

`IAM-01`, `AUT-01`, `INT-01`, `OPS-01` and `PLT-01` support the whole path.
`CTL-02` and the presentation components make it repeatable without
browser-route coupling. No arrow implies that the named portfolio component is
already integrated.

## Stepwise documentation sequence

The recommended review sequence protects the hardest-to-change boundaries
first while keeping each step small enough for founder review:

1. common interaction semantics: `C-001` to `C-006`;
2. `IAM-01` identity, trust and synthetic sessions: `I-001` to `I-005`;
3. `AUT-01` policy decision and authorisation: `AZ-001`;
4. `INT-01` interaction infrastructure and contract registry;
5. `CTL-01` Scenario Director: `D-001` to `D-004`;
6. `CTL-02` Presentation Gateway and `UX-04` Presentation Surface: `P-001` to
   `P-004`;
7. `UX-01`, `UX-03` and `UX-02` experience responsibilities;
8. `CNT-01` assets, content and source staging: `A-001` and `A-002`;
9. `KNO-01` knowledge, retrieval and evidence: `K-001` and `K-002`;
10. `DOM-01` engagement and domain records: `E-001`;
11. `WRK-01` work, case and workflow: `W-001`;
12. `RUL-01` rules, decisions and transformations: `R-001`;
13. `RPT-01` reports, visualisations and diagrams: `RP-001` to `RP-004`;
14. `AUD-01` and `ANA-01`: `AU-001`, `AN-001` and `AN-002`;
15. `AIO-01` and `ADP-01`: `AI-001` and `X-001`; and
16. `OPS-01` and `PLT-01`: `O-001`, `O-002`, `L-001` and `L-002`.

Each step updates this catalogue and the relevant component-to-contract map.
The first detailed step should define the common interaction semantics; the
identity and synthetic-session component follows immediately because its
environment trust boundary would be costly to retrofit.

## Maturity and approval

The component and contract catalogue is a working design inventory. It does not
claim implementation, integration, production readiness or portfolio-wide
adoption. A detailed specification becomes agreed only through founder review;
a material security, privacy, interoperability or deployment choice also
requires an architecture decision record.

Architecture Portal remains canonical for the cross-portfolio logical
blueprint. Public Purpose Lab owns this applied demonstrator profile and its
evidence. Open BPM may adopt the common interaction and analytics contracts
while retaining ownership of its work-specific lifecycle and schemas.

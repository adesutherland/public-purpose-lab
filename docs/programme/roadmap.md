# Roadmap

Status: Agreed direction

Last reviewed: 25 August 2026

This roadmap defines the enduring direction for Public Purpose Lab. It is
organised around business outcomes and evidence rather than fixed dates,
products or numbers of components.

The Lab will establish a human-led **Discovery and Improvement Service**,
supported by a **Service Evidence Workbench** and a repeatable **Scenario
Director**. Together they will help organisations understand fragmented
systems and guidance, govern improvement work, and produce evidence-linked
reports without obscuring human responsibility.

The initial roadmap establishes the smallest credible end-to-end service and
technology foundation. The future roadmap extends that foundation only when
demonstrated value, authority and operating evidence justify it.

## Terminology

| Term | Meaning |
|---|---|
| **Discovery and Improvement Service** | The human-led service through which practitioners help an organisation understand its systems, information, policies, risks and improvement opportunities. |
| **Service Evidence Workbench** | The practitioner and client-facing software for managing engagements, assets, discovery, analysis, governed work and evidence-linked reporting. |
| **Scenario Director** | The test and demonstration facility that prepares synthetic scenarios, controls time and faults, observes progress and verifies evidence. It does not own business decisions. |
| **Engagement** | A bounded body of discovery or improvement work with a stated purpose, participants, authority, assets, decisions and outputs. |
| **Asset** | A registered source used by an engagement, including a document, policy, guidance item, system description, interview record, data extract or external link. |
| **Evidence** | Traceable material connecting a source, action, transformation, rule, model-assisted step or human decision to a finding or report. |
| **Finding** | A reviewable statement about a gap, conflict, dependency, risk, change or opportunity. A generated finding remains proposed until accepted by an authorised person. |
| **Guidance drift** | A relevant change between versions of policy, guidance or another controlling source, together with its possible effect on assets, decisions or work. |
| **Report** | A versioned, human-released output whose findings, evidence, limitations and decision authority can be inspected. It is not a legal opinion or compliance certificate. |

These terms distinguish the service, its software and its demonstration
facility. They also keep source evidence, generated analysis and accountable
human decisions separate.

## Business proposition

The Discovery and Improvement Service will support a consistent journey:

1. establish the purpose, authority and boundaries of an engagement;
2. register, upload or link relevant assets;
3. preserve source versions, provenance, classifications and rights;
4. map systems, information, responsibilities and dependencies;
5. expose gaps, conflicts, unknowns and guidance drift;
6. retrieve and evaluate source-backed evidence;
7. assign findings and decisions to accountable people;
8. apply explicit rules or transformations where they add value;
9. produce evidence-linked reports; and
10. retain a complete record of what happened, why and under whose authority.

AI may assist discovery, extraction, comparison and drafting. It does not own
facts or consequential decisions. Generated claims remain distinguishable from
source material and accepted findings, and every released report retains an
accountable human decision.

## Authority and compliance boundary

The Lab provides mechanisms for better-governed compliance-related work. It can
organise sources, detect change, apply explicit controls, manage review and
retain evidence. It does not declare that an organisation is legally or
regulatorily compliant.

Every engagement distinguishes:

- **source authority** — who issued the source;
- **interpretation authority** — who may determine what it means in context;
- **action authority** — who may approve a change or release an output; and
- **technical evidence** — what the components loaded, found, generated,
  executed and recorded.

Where legal, regulatory, clinical or other professional interpretation is
required, an appropriately qualified organisation or partner must hold that
responsibility. The Lab may supply the supporting evidence and workflow without
assuming that authority itself.

## Operating model

The Service Evidence Workbench will support two complementary modes:

- an **installable local edition** for private practitioner work, efficient
  analysis and portable demonstrations; and
- a **Kubernetes-compatible hosted edition** for shared demonstrations,
  collaboration and production-like operational learning.

The two modes form one logical product. They share business concepts,
interfaces, policies, scenario assets and evidence expectations. Local work
must retain the same privacy, security, provenance and audit discipline as
hosted work.

AI and retrieval providers remain replaceable. The local edition may use local
models, separately authorised services or user-owned subscriptions through
supported provider interfaces. The hosted edition uses explicitly authorised
service credentials or model services. Personal credentials and allowances are
never treated as shared platform credentials.

## Initial roadmap

The initial roadmap creates a broad but deliberately basic vertical slice. It
must be capable of demonstrating real value without implying production or
regulatory readiness.

### Initial business demonstrations

1. **Charity systems discovery and reporting** — turn fragmented synthetic
   sources into a reviewable system map, governed findings and an
   evidence-linked report. This remains the first end-to-end path established
   by [ADR-0001](../architecture/decisions/0001-grow-architecture-from-scenarios.md).
2. **Policy and guidance drift** — load two controlled source versions,
   identify material changes, connect possible effects to assets or work, and
   produce a human-reviewed impact report.

Both demonstrations use synthetic data or suitable public material. They must
show uncertainty, conflicting evidence, human authority, failure and recovery
rather than presenting an idealised happy path.

### Initial business capabilities and logical foundations

| Business capability | Initial logical foundation |
|---|---|
| Manage discovery and improvement engagements | Service Evidence Workbench with bounded engagement, participant, purpose, status and output records. |
| Manage source assets | Content and document capability for registration, upload or linking, versioning, classification, rights, provenance, staging, retention and disposal. |
| Map an organisation | Domain records for systems, information, responsibilities, interfaces, dependencies, gaps and conflicts. |
| Query policies and other sources | Knowledge, retrieval and source-grounding capability that returns evidence with visible provenance, gaps and uncertainty. `crexx-rag` is the preferred first portfolio component to qualify for this role. |
| Govern findings and decisions | Basic work and workflow capability for ownership, review, approval, rejection, escalation, completion and history. |
| Apply explicit business logic | Versioned rules, decisions and transformations with defined inputs, results and explanations. cREXX is the preferred open implementation surface where it provides a clear advantage. |
| Connect components | Replaceable adapters and versioned commands, APIs and events with ownership, correlation and idempotency. |
| Own operational information | Clearly owned engagement, asset, finding, work, policy, report and audit records supported by appropriate database and content storage. |
| Protect people and workloads | Human identity, role-based authority, workload identity, least privilege, purpose context and managed secrets. |
| Use AI accountably | Replaceable generation and embedding capabilities, structured results, provider and model provenance, resource controls, abstention and human release. |
| Produce useful reports | Versioned report definitions, evidence-linked findings, stated limitations and accountable release. |
| Operate and recover | Health, logs, metrics, traces, support views, failed-work handling, backup, restore, reset and documented recovery. |
| Demonstrate and test | Scenario Director for synthetic setup, authorised commands, event observation, checkpoints, replay and controlled failure. |
| Build and run consistently | A small set of components packaged for local installation and Kubernetes-compatible hosting, without turning every logical responsibility into a separate service. |

The logical foundations align with the [Architecture Portal system
blueprint](https://architectureportal.org/blueprint). Public Purpose Lab applies
the blueprint through demonstrators and returns implementation evidence,
limitations and lessons.

### Initial source and evidence lifecycle

The Workbench supports one governed lifecycle for policies and other assets:

> register → acquire → validate → version → stage → retrieve → review → apply →
> report → retain or dispose

Each stage preserves ownership, status and provenance. Source material,
generated analysis, accepted findings and released reports remain distinct.

### Initial report set

- **Asset and source register** — what is held, where it came from, who owns it
  and how it is governed.
- **Discovery map** — systems, information, responsibilities, dependencies,
  gaps, conflicts and unknowns.
- **Evidence brief** — a bounded finding with supporting sources, uncertainty
  and review status.
- **Guidance drift and impact report** — source changes, possible effects and
  accountable review decisions.
- **Decision record** — the rule, transformation or human decision applied and
  its evidence.
- **Demonstration evidence pack** — the scenario, actions, events, failures,
  recoveries, measures, outputs and limitations.

### Initial completion evidence

The initial roadmap is complete when the Lab can demonstrate that:

- a bounded engagement runs end to end in local and hosted modes;
- every released finding can be traced to its sources, processing and human
  decisions;
- access controls apply to people and workloads;
- repeated, delayed, refused and failed work is visible and recoverable;
- malicious or malformed input, conflicting evidence and uncertain AI output
  are contained rather than silently accepted;
- the environment can be reset, backed up and restored;
- software health is distinguishable from business work state; and
- value, effort, provider use, operating cost and limitations are recorded.

### Initial boundaries

The initial roadmap does not include real personal or client-confidential data,
live charity or public-service integration, autonomous consequential decisions,
legal or regulatory certification, clinical decision-making, a production
multi-tenant service, full workflow-standards coverage, or replacement of an
organisation's core enterprise systems.

## Future roadmap

The future roadmap extends proven capabilities rather than expanding the
initial estate speculatively.

| Direction | Business outcome |
|---|---|
| Governed organisational engagements | Support bounded real-world work under agreed contractual, privacy, retention, security and operating arrangements. |
| Partner-backed policy services | Combine the Workbench's evidence and workflow with maintained policy content and accountable legal, regulatory or domain interpretation. |
| Continuous guidance monitoring | Monitor authorised sources, identify revisions, assess affected assets and coordinate review and change work. |
| Rich work and case management | Support longer-lived cases, queues, service levels, escalation, collaboration and proven process, case or decision interchange needs. |
| Broader integration | Add governed adapters for document repositories and operational systems only when a scenario supplies authority, ownership and a support model. |
| Multi-organisation service | Add organisational identity, delegated administration, tenant isolation and service management when a genuine shared-service need is proven. |
| Governed external reporting | Support approvals, signatures, submissions, delivery tracking, correction and retraction where the receiving authority is explicit. |
| Mature AI-assisted operation | Add durable, evaluated model and tool workflows with provider portability, resource governance, maker-checker controls and human work hand-offs. |
| Production-grade operation | Add availability, recovery, capacity, supply-chain, security-assurance and service-level capabilities in proportion to an authorised pilot. |
| Reusable open components | Publish stable contracts, packages, deployment assets, conformance tests and contribution routes after multiple scenarios prove the boundaries. |
| Further public-purpose scenarios | Use care disruption and rebooking, and later scenarios, to test whether the foundations genuinely transfer to other domains. |
| Regulated or clinical extension | Proceed only under separately approved governance, qualified leadership, independent assurance and explicitly authorised data and integrations. |

## How the roadmap advances

A capability advances when a scenario demonstrates useful behaviour and
produces sufficient evidence about ownership, privacy, security, failure,
recovery, cost and operability. Shared components require evidence from more
than one scenario.

Material architecture, privacy, security, licensing and scope choices are
recorded as architecture decision records. The exact initial architecture will
define component boundaries, interactions, information ownership, trust zones
and deployment views. The working
[initial implementation plan](implementation-plan.md) sequences delivery, tests
and evidence without changing the business direction established here.

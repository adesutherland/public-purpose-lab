# Demonstration scenarios and functional requirements

Status: Founder-approved functional baseline

Catalogue direction approved: 31 August 2026

Detailed requirements approved: 1 September 2026

Decision authority: Public Purpose Lab founders

Information profile: Synthetic only

## Purpose

This document defines what a Public Purpose Lab demonstration must visibly do.
It is the approved functional gate for M4 component, contract and storage
implementation. It replaces architecture-led sequencing with demonstrable user
journeys, named screens, observable component behaviour and testable evidence.

The first delivery objective is not to complete a logical architecture. It is
to let an authorised presenter start an environment, introduce a scenario,
place synthetic actors into the relevant portals, guide an audience through
real user actions, show processing across deployed components and inspect the
resulting events and evidence.

This is a synthetic development demonstrator. It is not evidence of legal
compliance, production readiness, organisational adoption or professional
authority.

## Demonstration model

A demonstration has four distinct kinds of action:

| Action               | Owner                                              | Meaning                                                                                                                         |
| -------------------- | -------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| Scenario control     | Authorised presenter through the Scenario Director | Starts, pauses, resumes, stops or resets an approved scenario and requests its next allowed step.                               |
| Presentation control | Director through semantic events                   | Selects an approved view and supplies bounded display context. It never sends browser routes, credentials or arbitrary scripts. |
| User action          | External or synthetic human through a portal       | Uploads, pastes, reviews, amends, accepts or rejects information using the target application's ordinary interface.             |
| Business processing  | Owning component                                   | Validates a command, changes its own state, records evidence and emits a fact, refusal or failure.                              |

The Director may arrange and observe these actions. It may not type on behalf
of a user, read or write another component's private state, or infer business
completion from a screen change.

## Roles and accounts

| Role                    | Account and session requirement                                                                    | Demonstration use                                                                           |
| ----------------------- | -------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| Environment operator    | Named external account with deployment and support authority; no synthetic identity.               | Starts and stops the environment and performs bounded recovery.                             |
| Presenter               | Named external account; local development adapter or Google OIDC according to environment profile. | Operates the Director and selects allowed scenario actions.                                 |
| Synthetic reviewer      | Scenario-defined actor, signed into the Workbench through an environment-scoped grant.             | Uploads or pastes sources, makes staging decisions, reviews findings and releases reports.  |
| Synthetic audience user | Scenario-defined actor, signed into an audience portal when interaction is required.               | Demonstrates an end-user portal without using a real person's account.                      |
| Component workload      | Environment-managed workload identity and event permissions.                                       | Consumes commands, owns processing and emits facts. It never substitutes for a human actor. |

### FR-ID-001 External account setup

The local profile must provide an unmistakable development-only login for the
presenter and operator. A managed hosted profile must use the approved external
identity binding and must not create Lab-managed passwords. The active account,
role, environment and authentication profile must be visible in the Director.

### FR-ID-002 Synthetic actor definition

Each scenario package must list every synthetic actor by stable display name,
role, target application and permitted scenario purpose. Reusing a display
name in another environment does not reuse an identity or session.

### FR-ID-003 Synthetic portal sign-in

The presenter must be able to request a synthetic sign-in for a selected actor
and target portal. The identity component must issue an environment- and
scenario-bound grant through the protected backend channel. The target portal
must validate it, establish its own application session and visibly show:

- synthetic actor name and role;
- target application and demonstration session;
- local-synthetic or managed trust profile; and
- expiry or termination state.

No signed grant, bearer token, private key or broker credential may reach the
browser.

### FR-ID-004 Session separation

One scenario may use different synthetic actors in different portals. A
synthetic reviewer in the Workbench and a synthetic audience user in another
portal are separate application sessions. Reset or stop must terminate the
scenario's synthetic sessions without terminating the presenter's external
account.

## Screen catalogue

The identifiers below name semantic views, not URLs. A frontend may reorganise
navigation without changing the scenario when it continues to support the same
view purpose and required context.

| View ID            | Surface              | Required visible content and actions                                                                                                 |
| ------------------ | -------------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `DIR-ENVIRONMENT`  | Director Console     | Environment name, runtime profile, trust profile, component readiness, presenter identity and start/stop guidance.                   |
| `DIR-CATALOGUE`    | Director Console     | Approved scenarios, purpose, maturity, estimated duration, actors, required components and known limitations.                        |
| `DIR-RUN`          | Director Console     | Current stage and step, next allowed actions, assigned actors, target surfaces, checkpoints, pause/stop/reset and explicit failures. |
| `PRES-INTRO`       | Presentation Surface | Scenario problem, synthetic organisation, actors, desired outcome, exclusions and current stage.                                     |
| `PRES-PROGRESS`    | Presentation Surface | Business-language progress, current component, latest conclusive outcome and unresolved issue; no infrastructure secrets.            |
| `WB-ENGAGEMENT`    | Workbench            | Engagement purpose, authority statement, participants, scope, synthetic-only classification and status.                              |
| `WB-SOURCE-INTAKE` | Workbench            | Upload and paste controls, link registration, source metadata, rights, provenance, classification and clear submission state.        |
| `WB-SOURCE-STATUS` | Workbench            | Quarantine, validation, staging and processing state for each immutable source version, including refusals.                          |
| `WB-QUERY`         | Workbench            | Bounded question, cited passages, provenance, conflicts, gaps, unknowns, provider identity and limitations.                          |
| `WB-REVIEW`        | Workbench            | Proposed finding, supporting and contrary evidence, reviewer identity and accept/amend/reject controls.                              |
| `WB-REPORT`        | Workbench            | Report preview, evidence manifest, limitations, version, release state and accountable release action.                               |
| `OPS-COMPONENTS`   | Operations Console   | Every required component instance, version, workload identity, readiness, last activity and safe failure summary.                    |
| `OPS-EVENTS`       | Operations Console   | Chronological commands, outcomes and facts with component, actor, correlation, causation, time and evidence references.              |
| `OPS-LOGS`         | Operations Console   | Filtered operational logs for the selected scenario and component, with credentials and protected content redacted.                  |

### FR-UI-001 Semantic view selection

The Director must request a view using a semantic view identifier, target
surface, demonstration session, step identifier, bounded display context and
expiry. It must not supply a route, executable content, HTML or script.

### FR-UI-002 Target-owned navigation

The target application must decide how to render or navigate to a supported
view. It must refuse an unsupported, expired, wrongly bound or unauthorised
request and emit the resulting presentation outcome.

### FR-UI-003 Human input remains human input

When a step requires pasting, uploading, editing or approving, the Director may
select the relevant screen and explain the task but must wait for the user to
perform it. The component event—not the cue—advances the business checkpoint.

### FR-UI-004 Manual navigation remains possible

An authorised user must be able to reach the same Workbench and operations
views through ordinary accessible navigation. Director control enhances a
demonstration but is not the only usable route through the application.

## Common component and event requirements

### FR-CMP-001 Deployed component skeleton

Every component required by an approved scenario must exist as a running
Kubernetes workload, or as an explicitly equivalent local container, before
its scenario is described as deployable. A planned catalogue entry or library
package is not a deployed component.

### FR-CMP-002 Minimum component behaviour

From its first deployment every backend component must:

1. expose liveness, readiness, version and capability information;
2. authenticate as its own workload to the event infrastructure;
3. publish a readiness or capability event after successful startup;
4. accept at least one scenario-relevant command or query;
5. validate authority, purpose, version and idempotency;
6. emit a conclusive fact, refusal or failure event;
7. expose its activity through the operations views; and
8. restart without silently duplicating accepted work.

### FR-CMP-003 Shared implementation, separate instances

Several early skeletons may use one configurable Rust component-host image,
but each deployed instance must have its own component identity, event
permissions, health status, configuration and state ownership. Shared code does
not permit one component to read or write another component's private state.

### FR-EVT-001 Event carriage

Components exchange commands and facts through the common event
infrastructure. They are not required to connect directly to every other
component. The event path must preserve message type and version, issuer,
audience, environment, demonstration session, actor or workload, purpose,
correlation, causation, idempotency, classification and observed time.

### FR-EVT-002 Visible event timeline

The operations console must show a privacy-minimised event timeline within two
seconds of an accepted event during a local demonstration. The timeline must
distinguish commands, transport acknowledgements, presentation outcomes,
business facts, refusals and failures.

### FR-EVT-003 Processing progress

A component performing work longer than two seconds must emit `started` and
terminal `completed`, `refused` or `failed` facts. It should emit bounded
progress only when the progress has real meaning. A repeating spinner is not
processing evidence.

### FR-EVT-004 Delivery behaviour

Duplicate and out-of-order delivery must be visible and safely handled. Exact
redelivery returns the prior semantic outcome; changed content under the same
idempotency key is refused. No demonstration checkpoint may depend on
exactly-once transport.

### FR-OPS-001 Logs and evidence

The presenter must be able to select a scenario, component and correlation
identifier and view relevant structured logs and evidence references. Source
content, credentials and security material must not be copied into general
logs.

## Deployable component baseline

The following instances constitute the initial demonstrator platform. This is
a deployment requirement, not a requirement for a separate codebase or
database product for every row.

| Deployable instance    | Logical responsibility         | Minimum first behaviour                                                                                   | Principal events visible in `OPS-EVENTS`                                                                   |
| ---------------------- | ------------------------------ | --------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| `scenario-director`    | `CTL-01`                       | Admit scenario, own session lifecycle, request sign-in and semantic views, observe checkpoints.           | `scenario.started`, `scenario.step.requested`, `scenario.paused`, `scenario.stopped`, `scenario.reset`.    |
| `presentation-gateway` | `CTL-02`                       | Register surfaces, deliver semantic view requests and record target-owned outcomes.                       | `surface.registered`, `view.requested`, `view.applied`, `view.refused`.                                    |
| `identity-broker`      | `IAM-01`                       | Report trust profile and establish or terminate bounded synthetic sessions.                               | `synthetic-session.established`, `synthetic-session.refused`, `synthetic-session.terminated`.              |
| `authorisation`        | `AUT-01`                       | Evaluate one explicit protected-action request and return permit, deny or indeterminate with obligations. | `authorisation.decided`.                                                                                   |
| `engagement`           | `DOM-01`                       | Create and return one bounded synthetic engagement.                                                       | `engagement.created`, `engagement.refused`.                                                                |
| `source-governance`    | `CNT-01`                       | Receive upload/paste/link metadata, quarantine, validate and stage an immutable version.                  | `source.received`, `source.quarantined`, `source.validated`, `source.validation-refused`, `source.staged`. |
| `knowledge-processing` | `KNO-01` with `AIO-01` adapter | Consume staged source, report real processing state and later execute a cited query.                      | `processing.started`, `processing.completed`, `processing.failed`, `query.completed`, `query.abstained`.   |
| `review-workflow`      | `WRK-01`                       | Create a review task and accept one named review decision.                                                | `review.requested`, `finding.accepted`, `finding.amended`, `finding.rejected`.                             |
| `reporting`            | `RPT-01`                       | Build a versioned preview and later record an authorised release.                                         | `report.previewed`, `report.released`, `report.release-refused`.                                           |
| `audit-evidence`       | `AUD-01`                       | Retain append-oriented event/evidence references and return a scenario reconstruction view.               | `evidence.recorded`, `evidence-unavailable` operational signal.                                            |
| `operations`           | `OPS-01`                       | Project component readiness, event timelines and redacted log access.                                     | `component.ready`, `component.not-ready`, `operation.failed`.                                              |
| `event-infrastructure` | `INT-01`                       | Carry authenticated versioned messages and expose contract/capability compatibility.                      | Broker and delivery state is shown as operational evidence, not business fact.                             |

The Director Console, Presentation Surface, Workbench and Operations Console
are required frontend surfaces. They may initially be built and served from a
common frontend image while remaining independently addressable semantic
surfaces.

`crexx-rag` is the first knowledge-processing implementation to qualify, but
DS-03 initially requires only an honest processor that consumes a staged text
source and reports its lifecycle. `crexx-rag` becomes necessary for DS-04. It
does not own engagement, source, authorisation, review or report state.

## DS-01: Environment and identities

### Audience outcome

The audience can see what environment is running, which components are ready,
how real and synthetic identities differ and how a synthetic actor is placed
into one portal without sharing a presenter's credentials.

### Script

| Step | Presenter or user action                                 | Required system behaviour                                                                                               | Visible proof                                                                                                |
| ---- | -------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| 1    | Operator starts the local or hosted environment.         | Platform starts event infrastructure and all required component instances.                                              | `DIR-ENVIRONMENT` and `OPS-COMPONENTS` show the environment and each readiness state.                        |
| 2    | Presenter signs into the Director.                       | Director validates the external application session and role.                                                           | Named presenter, environment and login profile are visible.                                                  |
| 3    | Presenter opens the scenario catalogue.                  | Director lists only admitted scenarios whose minimum components and trust profile can be evaluated.                     | `DIR-CATALOGUE` shows ready, unavailable or degraded status with reasons.                                    |
| 4    | Presenter creates the selected Demonstration Session.    | Director creates a new session and correlation scope without creating a synthetic user session.                         | `scenario.started` or a safe refusal appears in `OPS-EVENTS`.                                                |
| 5    | Presenter assigns `synthetic-reviewer` to the Workbench. | IAM authorises and delivers a backend-only environment- and session-bound grant; Workbench establishes its own session. | Workbench shows actor, role, trust profile and expiry; timeline shows establishment without secret material. |
| 6    | Presenter terminates or resets the session.              | Synthetic Workbench session terminates; presenter remains signed in.                                                    | Workbench reports termination and the event timeline records the owner and reason.                           |

### Acceptance

- No browser receives a grant, broker credential or private key.
- The same synthetic display name in a second environment creates a different
  trust and session identity.
- An unready identity component prevents actor placement but does not invent a
  ready scenario.

## DS-02: Scenario introduction and portal orchestration

### Audience outcome

The Director can explain and sequence a demonstration across screens without
fragile URL control, while each portal remains an ordinarily usable
application.

### Script

| Step | Presenter or user action                                         | Required system behaviour                                                                                                   | Visible proof                                                                               |
| ---- | ---------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| 1    | Presenter selects **Show introduction**.                         | Director emits a bounded `PRES-INTRO` view request to the registered presentation surface.                                  | Presentation screen shows problem, synthetic organisation, actors, outcome and limitations. |
| 2    | Presenter selects **Open engagement context**.                   | Director requests `WB-ENGAGEMENT`; Workbench resolves its own view for the signed-in reviewer.                              | Workbench changes view and preserves the actor/session banner.                              |
| 3    | Presenter selects **Open source intake**.                        | Director requests `WB-SOURCE-INTAKE` with engagement reference only.                                                        | Upload/paste controls are ready; no URL or DOM instruction appears in the event.            |
| 4    | User navigates manually to engagement and back to source intake. | Workbench provides equivalent accessible navigation without Director involvement.                                           | Same governed context is shown and no business event is falsely emitted.                    |
| 5    | Presenter pauses the scenario.                                   | Director stops issuing new automatic steps; existing sessions and in-flight component work remain governed by their owners. | `DIR-RUN` shows paused while component state remains truthful.                              |

### Acceptance

- An unsupported or expired view request is visibly refused.
- A view outcome satisfies only a presentation-progress checkpoint.
- Changing a screen cannot create an engagement, stage a source or complete
  processing.

## DS-03: File intake and visible processing

Status: First functional implementation scenario

### Audience outcome

A signed-in synthetic reviewer can upload a small synthetic text or Markdown
file, or paste equivalent text, and see it move through real deployed
components from receipt to completed processing while the event and operations
views explain what happened.

### Preconditions

- DS-01 and DS-02 acceptance paths pass in the selected environment.
- The scenario has one active synthetic-only engagement.
- `source-governance`, `knowledge-processing`, `audit-evidence` and
  `operations` report ready.
- The source fixture contains no real or confidential information and is no
  larger than the configured demonstration bound.

### Script

| Step | Presenter or user action                                                                  | Required system behaviour                                                                                                                       | Visible proof                                                                               |
| ---- | ----------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| 1    | Presenter shows `PRES-INTRO`, then selects **Begin source intake**.                       | Director requests `WB-SOURCE-INTAKE` for the active engagement.                                                                                 | Presentation explains the task; Workbench shows engagement and synthetic reviewer.          |
| 2    | Reviewer selects a `.txt` or `.md` file, or chooses **Paste text** and enters content.    | Workbench shows filename or paste mode, size, media type and a content preview before submission. No business component is called yet.          | User can cancel or correct metadata.                                                        |
| 3    | Reviewer supplies title, owner, rights, provenance and confirms synthetic classification. | Workbench validates required fields locally but does not claim authoritative acceptance.                                                        | Missing metadata is identified next to the relevant field.                                  |
| 4    | Reviewer selects **Submit to quarantine**.                                                | `CNT-01` validates session, authority, purpose, size, type and idempotency; stores an immutable version and emits receipt/quarantine facts.     | `WB-SOURCE-STATUS` shows received and quarantined; `OPS-EVENTS` shows correlated facts.     |
| 5    | Source component validates the acquired version.                                          | Validation checks supported media, non-empty content, digest and basic hostile/malformed input indicators. It emits validated or refused.       | Workbench shows each validation result and retained reason, not merely a spinner.           |
| 6    | Reviewer selects **Release to staging** for a validated version.                          | `AUT-01` and `CNT-01` evaluate the action; `CNT-01` records the named decision and emits `source.staged`, or refuses with reason.               | Actor, purpose, source version and staging outcome are visible.                             |
| 7    | `KNO-01` consumes the staged-source fact.                                                 | Processor records idempotent receipt, emits `processing.started`, performs the bounded initial text-processing step and emits terminal outcome. | `WB-SOURCE-STATUS` and `PRES-PROGRESS` show queued, processing and completed/failed states. |
| 8    | Presenter opens operations evidence.                                                      | Operations view filters events and logs by the scenario correlation identifier.                                                                 | Component sequence, timings, refusals and evidence references are inspectable.              |
| 9    | Presenter restarts `knowledge-processing`.                                                | Component reconciles accepted work and does not silently create a second completed result.                                                      | Readiness returns and replay/duplicate behaviour is visible.                                |

### DS-03 functional bounds

- Upload and paste support UTF-8 plain text and Markdown only in the first
  implementation.
- A linked HTTPS reference may be registered, but the backend must not fetch it
  in DS-03. It remains unresolved until a separate acquisition capability is
  approved.
- Submission progress may represent browser transfer; processing progress must
  come from component events.
- Source bodies remain in source-governance storage and are not copied into
  logs or presentation events.
- The initial processor may calculate digest, line/section counts and a safe
  text preview. It must not claim RAG ingestion or semantic understanding.

### Acceptance

- One upload and one paste follow the same component-owned quarantine,
  validation, staging and processing semantics.
- A malformed, empty or oversized source is refused and never reaches
  `knowledge-processing`.
- A user can view the entire correlated event sequence and relevant redacted
  logs from the UI.
- Restarting a component preserves conclusive state and idempotency.
- Director cues never count as receipt, validation, staging or processing.

## DS-04: Evidence query and uncertainty

### Audience outcome

The reviewer asks a bounded question over two staged, partly conflicting
synthetic sources and receives cited evidence that preserves conflict, gaps and
unknowns.

### Script

| Step | User action                                                 | Required system behaviour                                                                                                                     | Visible proof                                                               |
| ---- | ----------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------- |
| 1    | Reviewer opens `WB-QUERY` and enters the approved question. | Workbench identifies the engagement, staged source versions and intended purpose.                                                             | Query scope is visible before execution.                                    |
| 2    | Reviewer submits the query.                                 | `KNO-01` invokes the replaceable retrieval/provider adapter and records provider, configuration, inputs, resource use and abstention/failure. | Processing status appears in Workbench and operations timeline.             |
| 3    | Query completes or abstains.                                | Component returns cited passages, provenance, uncertainty and at least one visible conflict, gap or unknown.                                  | Every answer statement links to retained source evidence.                   |
| 4    | Reviewer opens a citation and contrary evidence.            | Workbench retrieves authorised evidence through the owning component.                                                                         | Source version, relevant passage and processing provenance are inspectable. |

### Acceptance

- Only staged immutable versions are queried.
- Unsupported claims remain absent or explicitly insufficient; they are not
  filled with plausible model text.
- `crexx-rag` is qualified here through the replaceable knowledge interface;
  its output is not treated as an accepted finding.

## DS-05: Accountable review and report

### Audience outcome

A named synthetic reviewer turns cited evidence into an accountable decision
and a versioned report preview without confusing generated text, evidence and
release authority.

### Script

| Step | User action                                               | Required system behaviour                                                                | Visible proof                                                                                 |
| ---- | --------------------------------------------------------- | ---------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| 1    | Reviewer selects **Propose finding** from a query result. | `WRK-01` creates a review item referring to evidence; proposed text remains unaccepted.  | `WB-REVIEW` distinguishes source evidence, generated proposal and current authority state.    |
| 2    | Reviewer accepts, amends or rejects.                      | Workflow validates reviewer role and records the exact decision and reason.              | Named actor, decision time, evidence and changed text are visible.                            |
| 3    | Reviewer requests a report preview.                       | `RPT-01` composes one versioned element from the accepted finding and evidence manifest. | `WB-REPORT` shows preview, limitations, version and unreleased state.                         |
| 4    | Reviewer selects **Release demonstration report**.        | Authorisation and reporting components record permit/refusal and accountable release.    | Released artifact and evidence manifest are inspectable; a refusal leaves preview unreleased. |

### Acceptance

- A proposal cannot become accepted without a named human action.
- A report cannot become released because a presentation or model step
  completed.
- The report states synthetic-only status, limitations and release authority.

## DS-06: Operations, failure and replay

### Audience outcome

The audience can inspect how the demonstrator behaves when a component is
unavailable, an event is duplicated or a scenario is reset.

### Script

| Step | Presenter or operator action                                         | Required system behaviour                                                                                    | Visible proof                                                      |
| ---- | -------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------ |
| 1    | Presenter opens `OPS-COMPONENTS`, `OPS-EVENTS` and `OPS-LOGS`.       | Operations component shows scenario-filtered state without exposing secrets or raw protected source content. | Component, event and log views share correlation identifiers.      |
| 2    | Presenter activates one allow-listed processing failure.             | Owning component contains the fault and emits an attributable failure.                                       | Workbench and presentation show failed/uncertain, never completed. |
| 3    | Operator restores or restarts the component.                         | Component reconciles inbox/outbox and exposes the recovery owner and result.                                 | Timeline shows failure, recovery and any safe retry.               |
| 4    | Presenter re-delivers an exact command and then a changed duplicate. | Exact duplicate returns prior outcome; changed duplicate is refused.                                         | Both cases are distinguished in the event view.                    |
| 5    | Presenter stops and resets the scenario.                             | Components reset only declared disposable state, retain evidence and terminate old synthetic sessions.       | Successor session is distinct; prior evidence remains inspectable. |

### Acceptance

- The fault cannot affect trust roots, another environment or real data.
- Reset is not a database wipe and does not turn uncertainty into success.
- Replay produces the same material business result without duplicating an
  accepted review or report release.

## Implementation gates

The founders approved this detailed document and component baseline on 1
September 2026. Gate A is active. Delivery proceeds by visible capability:

1. **Gate A — deployed component mesh:** all required skeleton instances run,
   authenticate, publish readiness and appear in the operations views.
2. **Gate B — DS-01 and DS-02:** identity, scenario introduction and semantic
   portal orchestration work end to end.
3. **Gate C — DS-03:** upload/paste, quarantine, validation, staging, processing
   status, logs and events form the first functional business path.
4. **Gate D — DS-04:** two-source `crexx-rag` ingestion and cited query are
   qualified through visible evidence and uncertainty.
5. **Gate E — DS-05:** human review and evidence-linked report preview/release
   complete the source-to-report path.
6. **Gate F — DS-06:** adverse operation, restart and deterministic replay
   provide bounded assurance evidence.

Each gate must leave a runnable environment and a user-facing acceptance
script. Internal schemas, storage or service separation do not complete a gate
unless they are necessary for and exercised by that script.

## Gate progress and show-and-tell evidence

Every gate must produce a maintained Markdown evidence report and a PDF
distribution copy before it can close. The report combines a progress report,
system-test record and show-and-tell. It must contain:

1. the gate objective, approved requirements and achieved acceptance status;
2. the inherited platform and scenario context that existed before the gate;
3. exact source revision, build identity, environment profile and walkthrough
   date;
4. the demonstrated actor, screen, command, event and component flow;
5. real screenshots captured while walking the evidenced build through the
   approved flow, with captions identifying the relevant step and visible
   result;
6. the screens, functions, rules, contracts and component behaviours added or
   changed by the gate;
7. representative event, operational and test evidence connecting screen
   activity to component-owned outcomes;
8. refusals, adverse cases, limitations, deferred work and unresolved risks;
9. what the gate establishes and what it does not establish; and
10. the next gate, its intended user-visible outcome and prerequisites.

Screenshots must come from the exact source revision and environment described
by the report. Mockups, design images and screenshots from another build may
illustrate intent but do not count as system-test evidence. Screenshots must
contain synthetic information only and must exclude credentials, tokens,
private endpoints and other protected material.

The Markdown source, screenshots and PDF belong under architecture
implementation evidence. The PDF must identify its canonical Markdown source,
be rendered to images and visually checked for legibility, clipping, alignment
and complete figures before publication. Automated test output supports the
report but cannot replace the walked-through screenshots.

Gate closure requires founder approval of the implemented flow and its evidence
report. A partial gate report may be published as in-development evidence, but
it must not describe the gate as complete.

## Approved baseline decisions

The founders approved on 1 September 2026:

1. DS-01 to DS-06 as the initial demonstration set and order;
2. the named Director, Presentation, Workbench and Operations screen catalogue;
3. DS-03 as the first functional business-process scenario after the deployed
   mesh and orchestration gates;
4. the initial deployable component baseline;
5. one configurable Rust component-host image for the initial backend skeleton
   instances, while preserving separate deployed identities and boundaries;
6. the working event names as sufficient to start the next bounded contract
   definitions; and
7. the mandatory gate progress and show-and-tell evidence report defined above.

Approval authorises Gate A and only the contract and architecture detail needed
to deploy, authenticate and observe the component mesh. Later evidence may
revise this baseline through the normal decision and review process.

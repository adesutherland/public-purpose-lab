# D-002: Scenario lifecycle

Status: Accepted M3.1 logical baseline; schema and implementation pending

Version: 0.1.0

Last reviewed: 27 August 2026

Governing decision:
[ADR-0011](../../../decisions/0011-establish-the-m3-scenario-control-invariants.md)

Owner: [`CTL-01`](../../components/ctl-01-scenario-director.md)

Semantic type: Demonstration Session command, authoritative state fact and
command outcome

Canonical schema: Not selected in M3.1

## Purpose

`D-002` creates and controls one Demonstration Session for an admitted `D-001`
package. It makes prepare, start, pause, resume, complete, stop and reset
requests explicit, attributable, idempotent and revisioned.

The contract controls scenario coordination only. It does not authenticate the
presenter, issue a synthetic sign-in grant, mutate business records, control a
browser, or report a domain outcome that belongs to another component.

## Participants and authority

| Role               | Participant                                            | Responsibility                                                                 |
| ------------------ | ------------------------------------------------------ | ------------------------------------------------------------------------------ |
| Requesting actor   | Authorised presenter or operator under a named action  | Requests a permitted transition for one session and expected revision.         |
| Requester          | Director Console backend or other authenticated client | Carries the initiating actor without substituting for it.                      |
| Owner and receiver | `CTL-01`                                               | Authorises, validates, serialises and durably decides the lifecycle operation. |
| Observers          | `UX-03`, `OPS-01`, scenario participants               | Consume safe current-state facts; cannot write lifecycle state.                |
| Dependent owners   | `IAM-01`, `CTL-02` and participating components        | Apply their own cleanup or readiness operations through separate contracts.    |

An operator may recover a failed control record only through a named recovery
action. Platform access does not confer ordinary presenter control. A package,
surface, cue or event-channel subscriber cannot initiate a lifecycle transition
without accepted authority.

## Contract variants

| Variant                             | Kind            | Meaning                                                                                      |
| ----------------------------------- | --------------- | -------------------------------------------------------------------------------------------- |
| `CreateDemonstrationSession`        | Command         | Binds a new session identifier to one admitted package version and environment.              |
| `PrepareDemonstrationSession`       | Command         | Evaluates prerequisites and requests transition from `preparing` to `ready`.                 |
| `StartDemonstrationSession`         | Command         | Starts a currently ready session.                                                            |
| `PauseDemonstrationSession`         | Command         | Stops new automatic Director actions while leaving protected time and in-flight work active. |
| `ResumeDemonstrationSession`        | Command         | Resumes a paused session after current readiness and authority checks.                       |
| `CompleteDemonstrationSession`      | Command         | Records the authorised control-plane decision that the run is complete.                      |
| `StopDemonstrationSession`          | Command         | Terminates further Director actions and begins bounded dependent cleanup.                    |
| `ResetDemonstrationSession`         | Command         | Requests `D-003` reset coordination and a distinct successor session.                        |
| `DemonstrationSessionStateChanged`  | Fact            | Reports an accepted lifecycle transition and new authoritative revision.                     |
| `DemonstrationSessionStateSnapshot` | Query result    | Reports the current safe lifecycle view and revision.                                        |
| `ScenarioLifecycleOutcome`          | `C-003` outcome | Reports acceptance, refusal, expiry, duplication or failure of the requested operation.      |

`ResetDemonstrationSession` is the lifecycle entry point; its target operations
and partial outcomes are governed by `D-003`. A reset request is not accepted as
complete merely because reset work began.

## State model

| State        | Allowed next control states                | Meaning                                                                         |
| ------------ | ------------------------------------------ | ------------------------------------------------------------------------------- |
| `preparing`  | `ready`, `failed`, `stopped`               | Admission is fixed but current prerequisites are being evaluated.               |
| `ready`      | `running`, `failed`, `stopped`             | Required readiness is currently satisfied.                                      |
| `running`    | `paused`, `completed`, `stopped`, `failed` | New authorised scenario actions may be coordinated.                             |
| `paused`     | `running`, `stopped`, `failed`             | No new automatic scenario action is issued.                                     |
| `completed`  | `superseded`                               | The run is complete; only reset to a successor or retained inspection remains.  |
| `stopped`    | `superseded`                               | The run was stopped; only reset to a successor or retained inspection remains.  |
| `failed`     | `superseded`                               | Safe continuation is unavailable; reset requires resolved ownership and policy. |
| `superseded` | none                                       | Successful reset produced a new Demonstration Session.                          |

Preparation may be repeated while the session remains `preparing`. Loss of a
runtime prerequisite after `ready` is reported by `D-004` and may block start
or resume; it does not silently edit history. A material loss while running may
cause an authorised fail or stop transition according to package and component
policy.

The baseline permits reset only from `completed`, `stopped` or `failed`. A
running or paused session must accept a separately attributable stop before
reset begins; package content cannot weaken this rule. The existing session
becomes `superseded` only after every required `D-003` reset target reaches a
conclusive successful result and the successor record is durably created. No
successor is reported ready on partial or uncertain reset.

## Command information

Every lifecycle command supplies or resolves:

- exact contract and command version;
- environment and Demonstration Session identifiers;
- admitted package identifier, version and digest;
- requested action and stable operation identifier;
- expected current lifecycle state and revision;
- requester and initiating actor through `C-002`;
- purpose, authority, constraints and policy reference;
- issue, expiry, correlation, causation and idempotency information;
- safe human reason where the action requires one; and
- requested evidence and recovery profile.

A reset request additionally identifies the accepted `D-003` reset-plan
reference. It cannot supply arbitrary reset instructions.

## State fact and snapshot information

The authoritative state fact records:

- session, environment, package and revision;
- previous and new state;
- accepted operation and safe outcome reference;
- effective operational time and optional scenario logical-time context;
- initiating principal types and privacy-minimised references;
- policy, capability, correlation and evidence references; and
- successor or predecessor session reference where applicable.

The snapshot is a projection of accepted facts. It includes current readiness
summary and incomplete-operation status but excludes raw commands, credentials,
grants, browser state, routes and private component records.

## Preconditions and transition rules

All commands require:

- authenticated requester and, where applicable, attributable initiating
  actor;
- current authority for the named action, environment and session;
- exact target, audience, purpose and information profile;
- compatible package and contract versions;
- matching expected lifecycle revision;
- operation within its validity window; and
- sufficiently healthy control state to decide durably.

Start and resume additionally require current `D-004` readiness. Completion
requires the package's declared control-plane completion conditions and an
authorised decision; this is not permission to invent a business fact. Stop is
available as a safe containment action under its own authority even when normal
progress is unavailable.

## Pause, stop and complete semantics

Pause prevents `CTL-01` from issuing new automatic package steps, business
commands and presentation cues. It does not:

- cancel or roll back already accepted component work;
- stop delivery of existing facts or outcomes;
- freeze operational, security, certificate, grant or application-session
  time;
- keep a surface, actor or grant valid; or
- suspend required evidence and operational monitoring.

Stop refuses new scenario actions and requests separately governed termination
or deregistration where declared. Cleanup outcomes remain visible. Stop never
claims that component-owned business or evidential state was erased.

Complete is an attributable statement about the scenario run. It may require
declared checkpoints, but it does not convert those observations into new
domain facts or a released report.

## Reset and successor identity

Reset retains the original session record and evidence. Successful reset:

1. obtains conclusive owner outcomes for all required reset targets;
2. closes or invalidates old scenario-dependent surface and synthetic-session
   bindings through their owning contracts;
3. marks the prior session `superseded`;
4. creates a successor with a new Demonstration Session identifier; and
5. re-evaluates package admission and readiness for the successor.

The successor may use the same package, fixture and actor display names, but it
does not inherit grants, application sessions, surface registrations,
idempotency operations or uncertain commands from the prior session.

Environment identity and trust-domain recovery are not scenario reset. A new
environment trust domain always requires new bindings under M2.

## Common-envelope and outcome rules

Commands use `C-001` and `C-002`; command results use `C-003`; retained evidence
is linked through `C-004`. `DemonstrationSessionStateChanged` is emitted only
after the authoritative transition is durable. An accepted `C-003` outcome is
not used as a substitute for that fact.

The state fact never carries the original authority context in full. It retains
safe attribution and evidence references appropriate to its audience.

## Idempotency, concurrency and ordering

- Idempotency is scoped to environment, session, lifecycle action and semantic
  operation.
- A repeated identical command returns or references its conclusive outcome and
  cannot advance the revision twice.
- The same idempotency key with changed content is refused.
- Expected state and revision provide optimistic concurrency; a stale command
  is refused with the current safe revision reference.
- One accepted lifecycle transition produces one new monotonic session
  revision.
- Delivery order is not trusted. A delayed start cannot reopen a stopped,
  completed, failed or superseded session.
- A lost acknowledgement is reconciled using the original operation before any
  new action is attempted.

The first implementation may enforce one active lifecycle writer per session.
Readers and event consumers still tolerate repeated, delayed and out-of-order
facts.

## Refusal, failure and recovery

Safe reason classes include:

- `session-unknown-or-unavailable`;
- `package-not-admitted`;
- `presenter-or-workload-not-authorised`;
- `state-transition-not-permitted`;
- `expected-revision-mismatch`;
- `session-not-ready`;
- `dependency-state-indeterminate`;
- `expired`;
- `idempotency-conflict`;
- `successor-already-created`;
- `reset-incomplete-or-uncertain`; and
- `control-state-unavailable-or-corrupt`.

After restart, `CTL-01` reconstructs the last durable revision. An operation
without a conclusive record is reconciled; it is not blindly reissued or
reported accepted. Corrupt or unavailable state makes the session not ready and
names the owner of recovery.

## Audit, privacy and analytical use

Evidence records the session and package references, transition, revisions,
principal types and safe references, purpose, policy, result, operational time,
correlation and `C-004` links. It excludes credentials, raw identity assertions,
grants, cookies, internal routes, full business payloads and reusable session
values.

Analytics may measure lifecycle duration, pauses, safe refusal classes,
recovery and reset success. An analytical write cannot cause a transition.

## Versioning and compatibility

`D-002` follows `C-006`. Changes to state meaning, allowed transitions,
authority, idempotency, revision semantics, reset succession or the distinction
between control and business completion are breaking.

Consumers refuse unsupported command and fact versions. Retained state facts
remain interpretable after a version is deprecated.

## Transport-neutral examples

An accepted example is an authorised presenter starting revision 2 of a ready
session. `CTL-01` durably records revision 3 as running, returns an accepted
outcome and emits one correlated state fact.

A negative example is a delayed resume for revision 5 arriving after an
authorised stop created revision 6. The receiver refuses the stale transition;
it does not infer intent from delivery order or reopen the session.

## Conformance evidence

Evidence must demonstrate:

1. every allowed and forbidden transition;
2. authenticated presenter and workload attribution without substitution;
3. exact expected-revision and idempotency handling;
4. duplicate, changed-content, delayed and out-of-order commands;
5. pause does not freeze protected time or hide in-flight component outcomes;
6. completion remains distinct from business completion and report release;
7. stop prevents delayed work from reopening the session;
8. restart and lost-acknowledgement reconciliation do not duplicate a
   transition;
9. reset produces a distinct successor only after conclusive required target
   outcomes; and
10. lifecycle facts, snapshots, logs and analytics disclose no route,
    credential, grant or usable session value.

## Open implementation decisions

M3.1 does not select the state store, transaction mechanism, API, transport,
lock or leader profile, presenter authentication, event publication binding,
retention store or timeout values. Those decisions follow logical-contract and
threat-model approval.

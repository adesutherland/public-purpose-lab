# D-003: Reset, clock and fault control

Status: Accepted M3.1 logical baseline; schema and implementation pending

Version: 0.1.0

Last reviewed: 27 August 2026

Governing decision:
[ADR-0011](../../../decisions/0011-establish-the-m3-scenario-control-invariants.md)

Owner: [`CTL-01`](../../components/ctl-01-scenario-director.md) for
coordination; every target component for its own reset, logical-time and fault
capability

Semantic type: bounded test-control command, target outcome and control fact

Canonical schema: Not selected in M3.1

## Purpose

`D-003` coordinates three distinct assurance controls:

1. returning component-owned synthetic scenario state to a declared baseline;
2. supplying explicit logical scenario time to opted-in synthetic behaviour;
   and
3. activating and clearing named, bounded failure profiles.

These controls make demonstrations and adverse tests repeatable without giving
the Scenario Director arbitrary administrator, database, infrastructure or
security authority.

## Participants and authority

| Role             | Participant                                      | Responsibility                                                                                                |
| ---------------- | ------------------------------------------------ | ------------------------------------------------------------------------------------------------------------- |
| Requesting actor | Authorised presenter or operator                 | Requests a package-approved control for one session and purpose.                                              |
| Coordinator      | `CTL-01`                                         | Validates scope, sequences target requests, retains outcomes and refuses unsafe continuation.                 |
| Capability owner | Participating component or bounded test adapter  | Publishes, authorises, applies and evidences its own allow-listed capability.                                 |
| Platform owner   | `PLT-01`                                         | May implement specifically approved environment controls; platform access alone does not authorise their use. |
| Observer         | `OPS-01`, `AUD-01`, authorised assurance readers | Receives safe control state and evidence, never arbitrary control access.                                     |

Reset, logical-clock and fault actions have separate authorities. Permission to
reset a scenario does not imply permission to inject a fault, change time,
recover security state or operate a platform.

## Contract variants

| Variant                      | Kind           | Meaning                                                                                        |
| ---------------------------- | -------------- | ---------------------------------------------------------------------------------------------- |
| `ApplyScenarioReset`         | Command        | Requests one declared component-owned reset operation.                                         |
| `ScenarioResetTargetOutcome` | Target outcome | Reports conclusive success, refusal, expiry, duplicate, failure or uncertainty for one target. |
| `ScenarioResetSummary`       | Control fact   | Reports whether all required targets permit successor-session creation.                        |
| `SetScenarioLogicalTime`     | Command        | Establishes an explicit logical instant or progression for an opted-in target.                 |
| `AdvanceScenarioLogicalTime` | Command        | Advances supported logical scenario time by a declared bounded operation.                      |
| `ScenarioLogicalTimeChanged` | Control fact   | Reports the accepted logical-time revision separately from operational time.                   |
| `ActivateScenarioFault`      | Command        | Activates one named, allow-listed adverse-case profile.                                        |
| `ClearScenarioFault`         | Command        | Clears the named fault or reconciles its automatic expiry.                                     |
| `ScenarioFaultStateChanged`  | Control fact   | Reports activation, expiry, clear, refusal or failure without exposing unsafe detail.          |

Each command has its own `C-003` outcome. A target success is not inferred from
delivery, elapsed time or a changed presentation.

## Capability declaration

Every target publishes a semantic capability before it can be referenced by a
`D-001` package. The declaration includes:

- capability identifier and exact version;
- owner, target and supported environment/information profiles;
- control kind: reset, logical time or fault;
- accepted scope and parameters;
- allowed session states and authority requirements;
- preconditions, maximum duration and automatic safety bounds;
- expected facts, outcomes and evidence;
- safe recovery and support owner; and
- limitations and conformance evidence.

Capability identifiers describe intent such as `synthetic-fixture-baseline` or
`source-adapter-timeout`; they are not shell commands, URLs, deployment names,
database tables or vendor configuration.

## Reset semantics

### Component ownership

A reset plan is a dependency-aware list of required and optional target
capabilities. `CTL-01` coordinates it, but each component:

- validates current authority, session, package and target scope;
- owns the baseline and the state it may change;
- reconciles duplicate or uncertain reset operations;
- retains substantive history and evidence according to its policy; and
- reports a conclusive target outcome or explicit uncertainty.

No target accepts a generic command to clear its database or filesystem. A
reset adapter exposes only the minimum semantic operation needed for the
approved synthetic fixture.

### Retained and replaceable state

The reset plan distinguishes:

- scenario fixture state that may be recreated;
- accepted business or test facts that are retained as historic evidence;
- application and surface sessions terminated through their owners;
- replay, revocation, idempotency and security state that is reconciled rather
  than erased; and
- environment identity, roots and signer configuration, which are outside
  scenario reset.

Reset never silently deletes the evidence needed to explain the preceding run.
Where a component creates a clean fixture generation, the prior generation and
reset relationship remain attributable through safe references.

### Success, partial failure and successor session

`ScenarioResetSummary` is successful only when every required target has a
conclusive successful result and no target remains uncertain. Optional target
failure is visible and may still prevent readiness according to the package.

An active session must first reach an accepted `D-002` terminal state. Reset
cannot race a running or paused Director by implicitly stopping it.

On success, `D-002` creates a distinct successor Demonstration Session and the
old session becomes superseded. On partial failure, no successor is reported
ready. Recovery never treats a rerun as permission to repeat a possibly
accepted target reset without reconciliation.

Environment backup and restore remain governed by the framework recovery
domains. If same-environment security continuity cannot be proved, recovery
creates a new environment identity and synthetic trust domain; a scenario reset
cannot repair or override that decision.

## Logical-time semantics

Scenario logical time is test data, not a privileged clock. A logical-time
context declares:

- Demonstration Session and package scope;
- owner and opted-in target capability;
- logical instant, zone/calendar interpretation where required, and revision;
- progression mode supported by the target;
- operational issue and acceptance times;
- maximum permitted movement and package bounds; and
- correlation and evidence references.

The initial logical contract permits setting a declared scenario instant and
advancing it through an owner-supported bounded operation. A target that cannot
unambiguously separate logical from protected operational time refuses the
capability.

Logical time must never determine or extend:

- certificate, key, trust, revocation or rotation validity;
- message issue, expiry, replay or idempotency retention;
- grant, token, cookie or application-session validity;
- authorisation-decision freshness;
- evidence-recording, audit or security-event time; or
- platform health and timeout decisions.

Every fact that uses logical time also records or references observed
operational time and labels the logical context. Pause does not pause either
protected clocks or an independently advancing target unless its declared
logical-time operation says so.

## Fault-control semantics

A fault profile is a target-owned, named adverse behaviour with explicit
containment. Its declaration and activation state include:

- environment, Demonstration Session, package and target;
- capability and profile version;
- fault class and safe description;
- permitted parameters selected from a closed schema;
- activation operation, start, maximum duration and expiry;
- affected synthetic capability and expected observation;
- automatic and manual clear semantics;
- current state and revision;
- recovery owner and evidence references; and
- exclusions that must remain unaffected.

M3 fault profiles may model bounded outcomes such as delay, temporary
unavailability, duplicate delivery or explicit component refusal where the
owner can contain and observe them. They may not:

- execute arbitrary code, shell, SQL or network policy;
- expose or rotate keys, credentials, grants or sessions;
- weaken identity, authorisation or audit controls;
- affect another environment or undeclared target;
- process real or confidential data;
- permanently destroy substantive evidence; or
- remain active without a bounded expiry or explicit recovery path.

Fault activation does not authorise the scenario action whose failure it
simulates. The target continues to validate all ordinary commands.

## Command information

Every `D-003` command supplies or resolves:

- exact contract, capability and profile version;
- environment, package, session and target identifiers;
- control kind and allow-listed semantic operation;
- expected target control-state revision;
- requester, actor, purpose, authority and policy reference;
- closed, bounded parameters;
- issue, expiry, idempotency, correlation and causation;
- required evidence and safe recovery owner; and
- for reset, target-plan position and required/optional classification.

Commands contain no credentials, raw security material, routes, deployment
handles or arbitrary instructions.

## Preconditions and acceptance

The coordinator and target validate:

- active environment and compatible synthetic information profile;
- admitted package and permitted Demonstration Session state;
- authenticated requester and actor authority for the exact control;
- target ownership, capability, version and expected revision;
- parameters within the published closed bounds;
- expiry, containment, dependencies and recovery readiness; and
- no conflict with a currently active or uncertain operation.

The target's accepted outcome means only that its named control operation was
durably applied. The coordinator derives the reset summary or scenario
readiness separately.

## Idempotency, ordering and uncertainty

- Each target control uses an idempotency key scoped to environment, session,
  target, capability and semantic operation.
- Duplicate identical content returns or references the original outcome.
- Changed content under the same key is refused.
- Expected revision prevents a delayed clock or fault command from replacing a
  newer state.
- Clear is idempotent and cannot reactivate an expired or cleared fault.
- Reset plans assume no global transport order; dependencies advance only from
  conclusive owner outcomes.
- An uncertain operation is reconciled with its owner before retry or successor
  session creation.

## Refusal and failure

Safe reason classes include:

- `control-not-authorised`;
- `capability-unsupported`;
- `profile-or-version-unsupported`;
- `session-state-incompatible`;
- `target-or-scope-mismatch`;
- `parameter-out-of-bounds`;
- `protected-time-boundary-violation`;
- `fault-containment-unconfirmed`;
- `conflicting-control-active`;
- `expected-revision-mismatch`;
- `expired`;
- `idempotency-conflict`;
- `target-outcome-uncertain`; and
- `recovery-not-ready`.

Unsafe or indeterminate requests fail closed. Diagnostics expose a safe reason
and owner, not infrastructure or security details useful for escalation.

## Audit, observability and analytical use

Evidence records the control kind, capability and version, session, target,
scope, safe parameters, revisions, actor types and safe references, policy,
operational and logical time where applicable, result, expiry, recovery and
correlation.

Operational views make active logical-time and fault state clearly visible.
They never include a secret, route, raw security identifier or arbitrary
control instruction.

Analytics may compare scenario behaviour with and without a named fault or
logical-time context. Analytics cannot activate, clear or reset anything.

## Versioning and compatibility

`D-003` follows `C-006`. Changes to scope, authority, reset retention,
successor-session rules, logical/protected time separation, fault containment,
automatic expiry or recovery semantics are breaking.

Unknown control kinds, parameters and authority-bearing fields are refused.

## Transport-neutral examples

An accepted reset example asks each owner to restore its declared synthetic
fixture baseline. All required owners return conclusive results, so `D-002`
creates a successor session with new bindings while the old run remains
inspectable.

An accepted fault example activates an owner-published temporary source-adapter
timeout for one synthetic target and bounded duration, observes the safe
failure, and clears or automatically expires it.

A refused clock example attempts to move scenario time backwards so an expired
sign-in grant becomes valid. `IAM-01` ignores scenario time and the target
refuses the boundary violation.

## Conformance evidence

Evidence must demonstrate:

1. control authorities are independent and target-enforced;
2. only declared semantic capabilities and closed parameters are accepted;
3. reset preserves required evidence and security-state continuity;
4. partial or uncertain reset prevents a ready successor;
5. duplicate, conflicting, delayed and restarted reset operations reconcile
   safely;
6. logical time is clearly labelled and cannot affect any protected expiry;
7. unsupported targets fail closed rather than falling back to host-clock
   manipulation;
8. fault activation is session-, environment-, target- and time-bounded;
9. fault clear, expiry and restart cannot leave an invisible active fault;
10. arbitrary code, routes, infrastructure controls and security-state changes
    are refused; and
11. evidence and diagnostics reveal neither secret material nor unsafe control
    detail.

## Open implementation decisions

M3.1 does not select reset adapters, target transactions, logical-clock
libraries, timer sources, fault mechanisms, transport, persistence or timeout
values. Each selected binding must document containment, platform permissions,
restart behaviour and profile-specific evidence in an ADR or implementation
specification.

# D-004: Readiness and checkpoint

Status: Accepted M3.1 logical baseline; schema and implementation pending

Version: 0.1.0

Last reviewed: 27 August 2026

Governing decision:
[ADR-0011](../../../decisions/0011-establish-the-m3-scenario-control-invariants.md)

Owner: Each source component for its observation; [`CTL-01`](../../components/ctl-01-scenario-director.md)
for scenario readiness and checkpoint evaluation

Semantic type: readiness observation, checkpoint evaluation and safe query
result

Canonical schema: Not selected in M3.1

## Purpose

`D-004` tells a presenter and assurance run whether prerequisites are currently
sufficient and whether declared scenario claims are supported by attributable
observations. It keeps software health, interaction readiness, scenario
readiness, presentation progress and business completion distinct.

The contract evaluates existing observations. It cannot mutate business state,
authorise an action, repair a dependency, approve a report or turn missing
evidence into success.

## Participants and authority

| Role              | Participant                                                                  | Responsibility                                                                                          |
| ----------------- | ---------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| Observation owner | Participating component, `OPS-01`, `CTL-02`, `IAM-01` or another named owner | Publishes a bounded, versioned observation about its own state or accepted fact.                        |
| Evaluator         | `CTL-01`                                                                     | Evaluates package-declared readiness requirements and checkpoints without changing the observed source. |
| Requester         | Authorised presenter, test harness or support reader                         | Requests a safe evaluation or snapshot for the permitted audience.                                      |
| Evidence owner    | Component retaining the supporting record                                    | Authorises evidence access and reports absence or disposal.                                             |
| Consumer          | `UX-03`, automation or assurance reporting                                   | Displays or tests the result without converting it into operational truth.                              |

Only the owning component can state its business fact or readiness. `CTL-01`
may determine whether that fact satisfies a scenario checkpoint; it cannot
replace or strengthen it.

## Contract variants

| Variant                        | Kind                    | Meaning                                                                                 |
| ------------------------------ | ----------------------- | --------------------------------------------------------------------------------------- |
| `ScenarioReadinessObservation` | Observation             | Reports one owner's bounded current prerequisite state.                                 |
| `EvaluateScenarioReadiness`    | Query-like command      | Requests evaluation of a named readiness set for one package and session revision.      |
| `ScenarioReadinessEvaluation`  | Result and control fact | Reports the derived scenario readiness and every requirement outcome.                   |
| `EvaluateScenarioCheckpoint`   | Query-like command      | Requests evaluation of one declared checkpoint from current attributable observations.  |
| `ScenarioCheckpointEvaluation` | Result and control fact | Reports the checkpoint status, basis, freshness and evidence references.                |
| `ScenarioAssuranceSnapshot`    | Query result            | Presents the current safe readiness and checkpoint view without becoming its authority. |

Evaluation requests use command controls because they consume resources and
may create retained evidence, but they cannot change the underlying business
or presentation state.

## Claim classes

Every requirement and checkpoint declares exactly one claim class:

| Class                   | Example                                                                                         | Authoritative source                                            |
| ----------------------- | ----------------------------------------------------------------------------------------------- | --------------------------------------------------------------- |
| `software-health`       | A component process is responsive and has no detected internal fault.                           | The component or `OPS-01` health contract.                      |
| `interaction-readiness` | A required contract, identity, policy or delivery dependency can safely accept work.            | The owning boundary and accepted capability/readiness contract. |
| `scenario-readiness`    | All package prerequisites for a lifecycle action are currently sufficient.                      | `CTL-01` evaluation of named owner observations.                |
| `presentation-progress` | A semantic cue was applied, refused, expired or failed at a registered surface.                 | `P-004` outcome through `CTL-02`.                               |
| `business-fact`         | An owning component accepted and reported a domain change or decision.                          | The relevant business contract and component.                   |
| `evidence-state`        | A required attributable evidence item exists, is classified and is accessible to the evaluator. | The evidence owner through `C-004` and later `AU-001`.          |

One class does not imply another. A healthy process may not be ready; a ready
scenario has not completed; an applied cue is not a business fact; an accepted
command outcome is not the eventual domain event; and a business fact is not a
released report unless its owner records that transition.

## Observation information

A `ScenarioReadinessObservation` declares:

- observation identifier and exact contract/version;
- environment, Demonstration Session and package scope where applicable;
- source component, capability and owner;
- claim class and stable subject identifier;
- status, safe reason and limiting conditions;
- observed operational time, validity or maximum age;
- optional scenario logical-time context, clearly separated;
- component state or evidence revision;
- policy, configuration and dependency versions relevant to the claim;
- correlation, causation and safe evidence references; and
- classification and permitted audience.

An observation is not reusable after its declared freshness, source revision,
session or environment scope no longer matches.

## Readiness requirements and evaluation

A `D-001` package declares required and optional readiness requirements. Each
states the source capability, claim class, acceptable statuses, version,
freshness, scope and evaluation rule.

Each requirement evaluates to:

| Status           | Meaning                                                                   |
| ---------------- | ------------------------------------------------------------------------- |
| `ready`          | A current, attributable observation satisfies the declared requirement.   |
| `not-ready`      | A current observation conclusively reports an unsatisfied requirement.    |
| `unknown`        | No current, compatible or authorised observation can establish the state. |
| `not-applicable` | The package's explicit condition excludes the requirement for this run.   |

Overall scenario readiness is `ready` only when every required requirement is
`ready` and no package-defined safety rule blocks the action. Optional
requirements and limitations remain visible. `unknown` never defaults to
ready.

A readiness result is a time-bounded evaluation, not a permit. The receiving
component and `AUT-01` path revalidate current authority and state when a
command arrives.

## Checkpoint definition and evaluation

Each package checkpoint declares:

- stable checkpoint identifier, title and claim class;
- exact subject, expected state and comparison rule;
- authoritative source component and contract/fact/evidence kind;
- required package, session, correlation and optional stage scope;
- supported version and observation freshness;
- prerequisite checkpoints, if any;
- evaluation window and treatment of absence;
- required evidence references and classification; and
- whether the checkpoint is mandatory for scenario completion.

Checkpoint evaluation statuses are:

| Status          | Meaning                                                                                     |
| --------------- | ------------------------------------------------------------------------------------------- |
| `pending`       | The evaluation window remains open and no conclusive result exists.                         |
| `satisfied`     | Current attributable observations meet the declared rule.                                   |
| `failed`        | Current attributable observations conclusively contradict or fail the rule.                 |
| `expired`       | The evaluation window ended without the required result.                                    |
| `not-evaluable` | Required source, version, authority, evidence or integrity is unavailable or indeterminate. |

`not-evaluable` is not softened into pending or success. A correction or later
fact creates a new evaluation revision; it does not silently replace the prior
record.

## Evaluation information

Every readiness or checkpoint evaluation records:

- evaluation identifier, definition version and monotonic evaluation revision;
- environment, package, session and relevant lifecycle revision;
- evaluator workload and initiating actor references where applicable;
- result status and stable safe reason;
- exact observation identifiers and source revisions used;
- operational evaluation time and separately labelled scenario logical time;
- freshness and window decision;
- policy, capability, correlation and causation references;
- `C-004` evidence references; and
- disclosure classification and audience.

The evaluation does not copy the source payload, raw identity assertion,
credential, route or restricted evidence content.

## Acceptance and refusal

The evaluator verifies:

- admitted package and matching Demonstration Session;
- exact readiness/checkpoint definition version;
- requester authority to evaluate or view the claim;
- supported source contracts and capability versions;
- source ownership, environment, session, subject and correlation;
- observation integrity, classification, freshness and revision;
- deterministic bounded comparison rules; and
- access to required evidence references without assuming their content.

Safe refusal or non-evaluable reasons include:

- `definition-or-version-unsupported`;
- `session-or-package-mismatch`;
- `requester-not-authorised`;
- `source-not-authoritative`;
- `observation-missing-or-stale`;
- `observation-scope-mismatch`;
- `observation-integrity-unconfirmed`;
- `evidence-unavailable-or-unauthorised`;
- `claim-class-mismatch`;
- `evaluation-rule-not-supported`; and
- `control-state-unavailable`.

Detailed source state is not disclosed to unauthorised consumers merely to
explain a failed check.

## Idempotency, ordering and revisions

- An evaluation request is idempotent for its package, session, definition,
  source-revision set and semantic operation.
- Duplicate identical requests return or reference the same evaluation.
- Changed content under the same idempotency key is refused.
- Later observations produce a new evaluation revision; prior evaluations
  remain attributable.
- Delivery order is not source authority. The evaluator uses declared source
  revisions, occurrence and observed times and current owner state.
- A late fact from a stopped or superseded session cannot satisfy a successor
  session checkpoint.
- Re-evaluation after restart reconstructs the exact source set or reports
  `not-evaluable`; it does not fabricate continuity.

## Presentation and business-state separation

Presentation progress can be a checkpoint only when the package explicitly
declares a `presentation-progress` claim. It is then satisfied by the `P-004`
outcome for the named registered surface and cue, not by inspecting browser
history or a route.

A business checkpoint requires the owning component's accepted fact or
authorised evidence. `P-004`, an HTTP response, console state, elapsed time or
`C-003` command acceptance cannot satisfy it. Conversely, a business fact does
not prove that an audience-facing surface displayed it.

## Failure and recovery

If an observation source or evidence owner is unavailable, the evaluator
reports `unknown` or `not-evaluable` according to the definition. It does not
reuse stale state outside its declared window.

After restart, retained evaluations and their source references remain
immutable. New evaluations use current observations and create new revisions.
A corrupted evaluation store makes affected scenario readiness indeterminate
and identifies the recovery owner.

## Audit, privacy and analytical use

Evidence records the definition, claim class, result, safe reason, source
identifiers and revisions, freshness, policy, time, correlation and `C-004`
references. It avoids copying source content and limits identity and operational
detail to the permitted audience.

Analytics may calculate readiness duration, failure classes, checkpoint
progress and recovery time. Analytical projections cannot become observations,
evaluate operational state or cause lifecycle progress.

## Versioning and compatibility

`D-004` follows `C-006`. Changes to claim classes, source authority, status
meaning, default treatment of absence, freshness, evaluation rules or the
separation of presentation and business truth are breaking.

Unknown claim classes and evaluation rules are refused rather than interpreted
optimistically.

## Transport-neutral examples

An accepted business checkpoint observes a versioned fact from the component
that owns the synthetic review record, verifies the session and correlation,
and records `satisfied` with an opaque evidence reference.

An accepted presentation checkpoint records that the Workbench surface applied
the requested semantic review view. It remains a presentation-progress result
and does not claim that a review happened.

A negative example sees the correct-looking page after browser refresh but has
no matching `P-004` outcome or business fact. The relevant checks remain
pending or not-evaluable; the Director does not inspect the URL and infer
success.

## Conformance evidence

Evidence must demonstrate:

1. each claim class and status has one unambiguous meaning;
2. health, readiness, presentation, command acceptance and business completion
   cannot substitute for one another;
3. source ownership, version, session, correlation and freshness are enforced;
4. missing, stale, unauthorised and incompatible observations fail closed;
5. duplicate and restarted evaluation is idempotent and revisioned;
6. delayed prior-session facts cannot satisfy successor checkpoints;
7. corrections create new attributable evaluations without rewriting history;
8. a cue outcome can satisfy only an explicitly presentation-class checkpoint;
9. a business checkpoint requires an owning-component fact or evidence; and
10. snapshots, logs and analytics disclose no raw evidence, route, credential
    or usable session value.

## Open implementation decisions

M3.1 does not select the observation transport, query API, state store,
expression language, timer/freshness implementation, projection technology,
evidence store or user-interface binding. The first binding should use the
smallest deterministic comparison surface capable of the approved assurance
scenario; any executable rule engine requires separate bounded design and an
ADR.

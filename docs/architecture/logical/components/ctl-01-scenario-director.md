# CTL-01: Scenario Director

Status: Accepted M3.1 logical baseline; M3.3 reference implementation in development

Version: 0.1.0

Last reviewed: 27 August 2026

Governing decision:
[ADR-0011](../../decisions/0011-establish-the-m3-scenario-control-invariants.md)

## Purpose

`CTL-01` makes a synthetic business scenario repeatable, observable and safe to
demonstrate. It validates a versioned scenario package, owns the Demonstration
Session and its control lifecycle, coordinates separately authorised actions,
requests semantic presentation cues, evaluates checkpoints and assembles
privacy-minimised evidence references.

It is a control-plane component, not a domain authority. It never becomes the
owner of an engagement, source, workflow, finding, report, identity,
relationship, consent or application session merely because it coordinates or
observes one.

This specification defines the logical boundary. It does not select a runtime,
database, event broker, API, browser protocol, policy engine or deployment
topology.

## Accountable ownership

The Scenario Director owner is accountable for:

- the meaning and integrity of accepted [`D-001`](../contracts/demonstration/d-001-scenario-package.md)
  scenario packages;
- one authoritative control record for each Demonstration Session;
- authorised lifecycle transitions under
  [`D-002`](../contracts/demonstration/d-002-scenario-lifecycle.md);
- bounded reset, logical-clock and fault coordination under
  [`D-003`](../contracts/demonstration/d-003-reset-clock-and-fault-control.md);
- readiness and checkpoint evaluation under
  [`D-004`](../contracts/demonstration/d-004-readiness-and-checkpoint.md);
- correlation of commands, facts, presentation outcomes and evidence; and
- explicit refusal, uncertainty and recovery state.

The owner approves which packages and controlled-test capabilities may be used
in an environment. A presenter operates an approved scenario but cannot change
its security or business authority by editing presentation state.

## Non-responsibilities

`CTL-01` does not:

- authenticate principals, issue trust, sign grants or establish application
  sessions;
- decide shared access-control policy or override a deny, indeterminate result
  or receiving-component refusal;
- own or write another component's private records;
- convert a presentation cue, checkpoint or successful command outcome into a
  business fact;
- resolve browser routes or hold surface credentials;
- change an operating-system, certificate, grant, token, policy or audit clock;
- run arbitrary scripts, shell commands, database resets or network faults;
- erase substantive evidence or security history during scenario reset; or
- claim legal, clinical, regulatory, professional or production authority.

`CTL-02` owns presentation-surface registration and cue delivery. `IAM-01` and
`AUT-01` own their accepted identity and authorisation responsibilities. Each
participating component remains the policy-enforcement point and owner of its
business actions and facts.

## Principals and decision rights

| Principal or role          | Decision right and limit                                                                                                                                       |
| -------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Authorised presenter       | May request allowed lifecycle, reset, clock, fault and checkpoint actions for a named session. Possession of the Director Console is not sufficient authority. |
| Scenario Director workload | May validate packages and coordinate only the contracts, targets and purposes granted to `CTL-01`. It cannot substitute for the presenter or synthetic actor.  |
| Synthetic human actor      | Acts in an application session established through the M2 identity path. The actor is not the Director workload.                                               |
| Participating component    | Decides and applies its own business, reset or test action and emits the resulting fact or outcome.                                                            |
| Service owner              | Approves scenario-package admission, controlled-test capabilities and policy for the owned component.                                                          |
| Operator                   | May perform named support and recovery actions; operator access does not imply presenter or business authority.                                                |

Every protected action carries or resolves the requester, any initiating actor,
environment, Demonstration Session, purpose, target, permitted action and
constraints through `C-001` and `C-002`. `AUT-01` is consulted where the
accepted policy requires it. The receiver still enforces the result against
current state.

## Owned and referenced state

`CTL-01` owns the minimum control state needed to reproduce and explain one
scenario run:

- admitted package identifier, immutable version and content digest;
- Demonstration Session identifier, environment, lifecycle state and revision;
- predecessor or successor session reference created by reset;
- selected synthetic actor, surface-role and component-capability references;
- authorised presenter and Director workload references in privacy-minimised
  form;
- current stage and step, attempted operations and conclusive outcomes;
- scenario logical-time context and its revisions;
- reset and fault-control requests, target outcomes and recovery ownership;
- readiness observations, checkpoint definitions, evaluations and revisions;
- cue requests and redacted delivery outcomes, without routes or credentials;
- correlation, causation, idempotency and safe evidence references; and
- package, contract, policy and capability versions used for the run.

It references, but does not copy or own:

- application sessions and identity security state;
- component business records and domain events;
- substantive evidence, source content, reports and analytical artifacts;
- presentation-surface bindings and browser state; and
- platform secrets, trust roots, event-delivery internals or private storage
  locations.

References use `C-004` or another accepted opaque contract reference. A broken
or unauthorised reference becomes explicit uncertainty; it is not treated as
proof.

## Scenario package admission

Before a Demonstration Session becomes ready, `CTL-01` validates the `D-001`
package and records an admission decision. Validation includes:

1. contract and package version support;
2. immutable package digest and attributable provenance;
3. declared synthetic-only information profile and environment compatibility;
4. required actors, applications, surface roles, components and semantic
   capabilities;
5. separately identifiable lifecycle, business-command, cue, reset, clock,
   fault and checkpoint operations;
6. authority and purpose requirements for every protected action;
7. bounded evidence, retention and reset expectations; and
8. absence of credentials, private keys, raw grants, session values, browser
   routes, internal endpoints and arbitrary executable content.

Admission is descriptive, not authorisation to execute every declared action.
Current component readiness, identity, policy and receiving-component decisions
are evaluated again when the action is requested.

## Demonstration Session lifecycle

The authoritative lifecycle states are:

| State        | Meaning                                                                                                    |
| ------------ | ---------------------------------------------------------------------------------------------------------- |
| `preparing`  | Package admission and prerequisite discovery are incomplete.                                               |
| `ready`      | Required prerequisites are currently satisfied and the session may be started.                             |
| `running`    | The Director may issue the next authorised scenario actions and cues.                                      |
| `paused`     | The Director issues no new automatic scenario action; in-flight component work and security time continue. |
| `completed`  | The scenario's declared completion decision was accepted. This does not add business facts.                |
| `stopped`    | Further scenario actions are refused following an authorised stop.                                         |
| `failed`     | Safe continuation cannot be established; recovery ownership is explicit.                                   |
| `superseded` | A successful reset has created a new successor Demonstration Session.                                      |

`D-002` defines allowed transitions and outcomes. Every accepted transition is
durable, revisioned and attributable. A console view, presentation cue or
transport acknowledgement cannot change lifecycle state.

Pause is deliberately narrow. It does not freeze grants, certificates,
sessions, queues, in-flight domain work or operational clocks. Stop prevents
new Director actions and requests bounded dependent cleanup, but it does not
pretend that already accepted business work was undone.

## Reset and successor sessions

Reset is an authorised coordination operation under `D-003`, not a database
wipe or a reversal of history. Each target component owns an allow-listed reset
capability and returns an explicit outcome.

The baseline requires an active `running` or `paused` session to be stopped
before reset begins. This makes termination and reset separately attributable
and prevents new Director actions racing target cleanup.

On successful reset, the existing Demonstration Session becomes `superseded`
and `CTL-01` creates a new session with a new identifier. The package version
may remain the same and actor display names may repeat, but old grants, surface
bindings, idempotency scopes and delayed work do not become valid in the
successor. This proposal is intentionally visible for founder review because it
strengthens the accepted M2 Demonstration Session binding.

If any required target is refused, unavailable or uncertain, no successor is
reported ready. The old session and every reset outcome remain attributable,
and the owning component or operator is named for recovery.

Reset never recreates or clones an environment trust domain. Environment
recovery and security-state reconciliation remain governed by the framework
security model and M2 trust profile.

## Logical time

Scenario logical time is explicit test context supplied only to components that
declare support for it. It may let synthetic fixtures and scenario rules behave
deterministically, but it never changes or substitutes for:

- host, monotonic or protected validation clocks;
- certificate, grant, session, revocation or replay timing;
- message issue, receipt or evidence-recording time; or
- policy effective time unless an owning policy test adapter explicitly models
  a separate synthetic fact.

Every evidence record distinguishes observed operational time from scenario
logical time. Unsupported or ambiguous clock use is refused.

## Fault control

Faults are named, allow-listed test capabilities implemented and enforced by
their target owner. A package can request a fault profile; it cannot provide
arbitrary code or infrastructure instructions. Each activation is scoped to
one environment, Demonstration Session, target, capability and bounded time or
explicit clear operation.

M3 faults must not expose secrets, alter trust roots, grant new authority,
silently discard evidence, access real data or escape the synthetic scenario
boundary. An inability to prove containment makes the fault unavailable.

## Commands, facts and presentation cues

`CTL-01` distinguishes four paths:

1. lifecycle and demonstration-control commands owned by `CTL-01`;
2. business commands sent to the component that owns the requested change;
3. semantic presentation cues sent through `CTL-02`; and
4. observed facts and outcomes used for readiness and checkpoints.

A package step may correlate these paths but cannot merge their authority. A
successful cue says only that a surface applied a semantic view. A `C-003`
accepted outcome says only that the receiver durably accepted its command. A
business checkpoint is satisfied only by the declared authoritative fact or
evidence from its owner.

## Repetition, ordering and concurrency

- Every command has an idempotency key scoped to target, contract and semantic
  operation.
- Repetition of the same content returns or references the original outcome;
  reuse for different content is refused.
- Lifecycle commands include the expected session revision. A stale or
  conflicting transition is refused rather than silently reordered.
- Ordering is narrow to a session and its revisioned lifecycle or a target's
  declared aggregate. No global event order is assumed.
- Delayed work is revalidated against current session, trust, surface,
  authority, expiry and component state.
- `CTL-01` does not replay a business command, sign-in grant, cue or fault when
  its acceptance is uncertain. It reconciles the original operation first.

The first implementation may support one active control writer for a session,
but this is an implementation profile rather than a weakening of the
idempotency and revision invariants.

## Failure, restart and recovery

Control state must survive an ordinary Director restart. On recovery,
`CTL-01` reconstructs the last durable session revision and classifies each
incomplete operation as not issued, conclusively decided or uncertain.

Uncertain work is reconciled using the original operation identifier and the
owning component's safe outcome path. It is never converted to success because
a screen looks correct, and it is never blindly issued again. A missing or
corrupt control record makes the affected session not ready and requires named
recovery.

Surface reconnect and cue reconciliation belong to `CTL-02` and the `P-001` to
`P-004` contracts defined in M3.2. `CTL-01` retains only safe references and
does not recover browser routes or session credentials.

## Readiness and checkpoints

`D-004` keeps these claims distinct:

- component software health;
- contract and dependency readiness;
- scenario readiness;
- presentation delivery progress; and
- business or evidential checkpoint satisfaction.

`CTL-01` evaluates a checkpoint from versioned, attributable observations. It
reports `not-evaluable` when required evidence is absent, stale,
incompatible or unauthorised. It does not infer completion from elapsed time,
an HTTP success, a cue outcome or an operator assertion that lacks the declared
authority.

## Audit, privacy and analytics

Control evidence records package and contract versions, session revision,
principal types and safe identifiers, requested action, target, purpose,
outcome, policy and capability references, operational and logical time,
correlation and `C-004` references.

It excludes command payloads where unnecessary, raw identity assertions,
relationships, grants, tokens, cookies, keys, internal routes, source content
and reusable session values. Detailed refusal information is limited where it
could enumerate actors, surfaces or security configuration.

Analytics may measure scenario duration, transition outcomes, readiness,
checkpoint results, retries, faults and recovery. An analytical projection
cannot control the scenario or mutate operational truth.

## Deployment and replaceability

The logical component may initially share one Rust deployable with `CTL-02`,
and its console may use the common TypeScript frontend packages. Co-deployment
does not merge state ownership, authority, contracts or evidence.

The control-state store, interaction transport, API style, container runtime,
Kubernetes distribution and presentation binding remain replaceable. Local,
portable and Minikube profiles must preserve the same contract semantics and
make their different persistence, identity and recovery evidence explicit.

`CTL-01` can invoke cREXX where an inspectable, portable scenario rule or
transformation has demonstrated value. M3.1 selects no executable scenario
language: `D-001` is declarative and contains no arbitrary code. Adding an
executable rules surface requires bounded inputs, outputs, permissions and
resources, conformance evidence and an ADR; cREXX remains the preferred first
assessment under repository policy.

## Dependencies and replaceable bindings

The logical boundary depends on:

- `C-001` to `C-006` and `INT-01` for public interaction semantics;
- `IAM-01`, `AUT-01`, `I-002`, `I-004`, `I-005` and `AZ-001` for workload,
  presenter, synthetic actor and protected-action authority;
- `CTL-02` and `P-001` to `P-004` for presentation binding and outcomes;
- `O-001` and `O-002` for component readiness and controlled recovery; and
- `C-004` and later `AU-001` for evidence references and retained provenance.

M3.1 accepts none of the still-open physical bindings by implication.

## Conformance evidence required before implementation acceptance

Evidence must show that:

1. packages with unsupported versions, missing provenance, non-synthetic scope,
   hidden routes, secret material or undeclared capabilities are refused;
2. only an authenticated, authorised presenter and workload can operate the
   named session;
3. lifecycle revision and idempotency prevent duplicate or stale transitions;
4. Director restart cannot duplicate an accepted component action;
5. pause leaves protected clocks and in-flight owner work unaffected;
6. reset is component-owned, records partial failure and produces a distinct
   successor session only after required targets succeed;
7. delayed work, grants and bindings from the prior session cannot control the
   successor;
8. logical time cannot extend a grant, certificate, message or session;
9. fault injection is allow-listed, scoped, visible and recoverable;
10. cues cannot mutate business state and cue success cannot satisfy a business
    checkpoint;
11. readiness and checkpoints distinguish health, presentation and business
    completion; and
12. no control record, event, evidence view, browser history or diagnostic
    contains a credential, usable session value or internal route.

## Current limitations and decisions deferred

`CTL-01` now has an M3.3 Rust reference implementation with a closed bundled
scenario package, component-owned SQLite state, expected revisions, durable
inbox/outbox decisions, manual-step logical time, semantic reset coordination
and presentation checkpoint evaluation. Its executable source is
[`backend/components/ctl-01`](../../../../backend/components/ctl-01), with the
HTTP process adapter in
[`backend/apps/m3-runtime`](../../../../backend/apps/m3-runtime).

ADR-0017 to ADR-0020 record the current implementation bindings. The following
still require later founder review and, where material, ADRs:

- package signing, reviewed distribution and mutable-package governance;
- managed presenter authentication and Director workload binding;
- multi-replica transport, state and recovery qualification;
- managed persistence, retention, backup and restore qualification;
- external surface-session binding through `IAM-01`;
- additional controlled-time, reset or fault capabilities;
- retention and tamper-evidence binding for control evidence; and
- production availability, resource and deployment profiles.

No binding may weaken the accepted framework security invariants or broaden
the synthetic-only M3 scope without separate governance and qualification.

# Architecture direction

The architecture exists to support demonstrable public-purpose outcomes. It is
not a catalogue of fashionable infrastructure.

The agreed [conceptual architecture framework](framework-conceptual-overview.md)
brings the Service Evidence Workbench, Scenario Director, presentation surfaces
and common capabilities into one logical model. A shareable
[PDF edition](public-purpose-lab-conceptual-architecture.pdf) is available for
distribution.

The [logical architecture overview](logical/README.md) starts the next level of
design. It identifies every logical component and contract family and defines
the order in which each will be documented and reviewed.

The founder-approved
[demonstration scenarios and functional requirements](../scenarios/demonstration-scenarios-and-functional-requirements.md)
is the functional gate for the next architecture and implementation work. It
defines the required screens, deployed component instances, observable events
and acceptance scripts. Gate A is active; later M4 bindings follow its approved
visible-capability sequence.

The [Gate A component-mesh binding](gate-a-component-mesh.md) defines the
separate workload identities, working operational event shape, live Operations
projection and bounded acceptance evidence for the first implementation gate.

The accepted
[M3.3 runtime walking-skeleton baseline](m3-3-runtime-walking-skeleton-baseline.md)
binds the Director and presentation contracts to a minimal executable
composition. Acceptance establishes an implementation baseline, not an
implemented or qualified capability. The first implementation and its open
closure gates are recorded in the
[M3.3 evidence record](evidence/m3-3-runtime-walking-skeleton.md).
The accepted
[M3.4 identity and resilience baseline](m3-4-identity-and-resilience-baseline.md)
extends that composition with external-human application sessions,
backend-only synthetic sign-in, target-owned replay safety and a managed-hosted
adapter. Local implementation exists; hosted evidence and milestone closure
are now complete at in-development, synthetic-only maturity. The bounded
[local evidence](evidence/m3-4-local-identity-and-resilience.md) and
[managed-hosted evidence](evidence/m3-4-managed-hosted-identity-and-resilience.md)
record the result and its limits.

The [framework security model](security/framework-security-model.md),
[M1 threat model](security/m1-threat-model.md) and
[M2 threat model](security/m2-threat-model.md) define accepted cross-cutting and
bounded milestone baselines. The
[M3 Scenario Director threat model](security/m3-threat-model.md) is the accepted
M3.1 logical threat baseline; its physical bindings remain undecided. None is a
claim of production security qualification.
The [M3.2 presentation and hosted-binding extension](security/m3-2-presentation-threat-extension.md)
is the accepted binding threat baseline. Its infrastructure lifecycle is
evidenced separately; later M3.3 and M3.4 records implement and exercise its
presentation, application and managed-trust controls at development maturity.
The accepted
[M3.3 runtime-binding threat extension](security/m3-3-runtime-binding-threat-extension.md)
covers the package, state, time/reset/fault and deployable bindings; its bounded
native, container, Minikube and private hosted evidence is recorded.
The
[M3.4 identity and resilience threat extension](security/m3-4-identity-and-resilience-threat-extension.md)
records the implemented local controls and managed-hosted gates. Those bounded
gates have now passed at development maturity.
Results and remaining programme gates are recorded separately under
[architecture implementation evidence](evidence/README.md).

## Initial shape

The first implementation should be a thin, end-to-end walking skeleton with as
few deployable units as the scenario permits. Logical component boundaries may
precede physical service boundaries; a component becomes independently deployed
only when ownership, trust, scaling, resilience, or change evidence justifies it.

The intended foundation includes:

- **Rust backend components** for reliable service and integration behaviour;
- **a TypeScript web application** for participants, reviewers, and the
  presenter-facing Scenario Director;
- **Kubernetes-compatible packaging** for repeatable environments and later
  production-like testing;
- **explicit commands and domain events** with schemas, correlation,
  causation, idempotency, and compatibility rules;
- **independent data ownership** rather than an integration-wide shared
  database;
- **identity and zero-trust boundaries** for people, workloads, and
  organisations, with a visible local-synthetic or managed trust profile and
  fail-closed readiness when the profile is insufficient;
- **externalisable access-control policy decisions**, with receiving-component
  enforcement, authoritative relationship inputs, decision outcomes,
  obligations, policy versions and privacy-minimised evidence;
- **replaceable adapters** around external charity, NHS, and social-care
  interfaces;
- **end-to-end observability and evidence** across each business transaction;
  and
- **bounded AI assurance** with explicit release, abstain, refuse, and escalate
  outcomes.

## Scenario Director

The Scenario Director makes a business story repeatable. It may start, pause,
reset, and inspect a scenario; issue authorised business commands; observe
events; display evidence; and verify checkpoints.

It is not the owner of domain decisions. Components validate commands, apply
their policies, record decisions, own their state, and emit facts. The same
scenario definition should support a live demonstration, automated acceptance
testing, failure injection, and adversarial assurance.

## cREXX position

cREXX is available as a project asset, not a mandatory platform layer. A
component may use cREXX for portable rules, transformations, scenarios,
automation, or integration behaviour when it provides a clear advantage.

Each proposed use should answer:

1. What capability does cREXX provide more clearly or safely here?
2. What inputs, outputs, permissions, and resource limits define its boundary?
3. How will the asset be versioned, tested, observed, and deployed?
4. Can an operator understand failures without specialist knowledge?
5. What simpler alternative was considered?

## Required cross-cutting evidence

Each end-to-end path should retain enough evidence to reconstruct:

- who or what initiated the action;
- the command, source, purpose, and applicable policy version;
- relevant events, correlation, causation, and component decisions;
- data provenance and transformations;
- AI model or configuration identifiers where AI is involved;
- unknowns, refusals, retries, human approvals, and overrides; and
- the final released output or external action.

## Decision records

Lasting decisions live in [`decisions/`](decisions/README.md). A decision record
states context, decision, consequences, and evidence needed to revisit it.

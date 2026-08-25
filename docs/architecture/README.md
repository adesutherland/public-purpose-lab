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

The [framework security model](security/framework-security-model.md) and
[current threat model](security/m1-threat-model.md) define the cross-cutting
baseline against which those components and contracts are implemented. They are
versioned working baselines, not claims of production security qualification.
Bounded results and remaining acceptance gates are recorded separately under
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
  organisations;
- **externalised policy where valuable**, with decision inputs, outcomes, rule
  versions, and explanations retained;
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

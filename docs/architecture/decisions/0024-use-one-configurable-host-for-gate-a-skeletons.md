# ADR-0024: Use one configurable host for Gate A skeletons

Status: Accepted
Date: 2026-09-01

## Context

Gate A must deploy the complete initial component mesh before later scenario
gates add source, knowledge, review and reporting behaviour. Creating nine
unrelated service implementations merely to prove deployment and event
boundaries would add code volume without adding business evidence. Treating
the components as one process, however, would conceal the identity,
permissions, readiness and failure boundaries that Gate A is intended to test.

The founder-approved functional baseline explicitly permits a shared Rust
component-host image while requiring separate deployed instances.

## Decision

Use one configurable Rust `ppl-component-host` binary for the nine Gate A
skeleton instances not already implemented by the M3 runtime. Each instance
has its own component and instance identifier, NKey workload identity, broker
publish and subscribe permissions, configuration, health endpoints and
observed readiness.

The existing M3 runtime continues to own `CTL-01`, `CTL-02` and `IAM-01`. Those
three workloads publish the same operational readiness shape without moving
their existing responsibilities into the configurable host.

The working Gate A `O-001` v0.1.0 binding carries privacy-minimised readiness,
capability-probe commands and conclusive outcomes. This working binding does
not promote the planned logical `O-001` contract to an agreed canonical
contract. The Operations Console derives status only from events it actually
observes; its expected-component catalogue may identify missing instances but
cannot mark them ready.

The configurable handlers accept only bounded capability probes in Gate A.
They do not simulate engagement, source, knowledge, workflow, report or audit
state changes.

## Consequences

- Gate A can prove all twelve deployed identities and event paths with one
  maintainable implementation image.
- A shared binary does not weaken per-workload authentication or permissions.
- The first Operations projection is in-memory and rebuilds from recurring
  readiness events after restart.
- Probe idempotency is process-local because probes have no business side
  effect. Durable idempotency is still required when a later gate adds a
  state-changing handler.
- Components may split into separate code or images when ownership, scaling,
  resilience, data or change evidence justifies it.

## Validation and review

Gate A evidence must show twelve unique workload identities reporting ready,
nine bounded probe commands reaching the configurable instances, conclusive
correlated outcomes, missing or stale instances shown honestly, and recovery
after broker and workload restart. Revisit this decision before adding any
handler whose data ownership, dependency set or security posture cannot be
isolated by configuration and workload deployment alone.

# Architecture decision records

Use a short architecture decision record (ADR) for a choice that materially
affects component boundaries, data, privacy, security, interoperability,
operability, licensing, or the project's ability to change direction.

## Status values

- Proposed
- Accepted
- Superseded
- Rejected

## Template

```markdown
# ADR-NNNN: Decision title

Status: Proposed
Date: YYYY-MM-DD

## Context

What question must be answered? What evidence and constraints matter?

## Decision

What is being decided, including explicit boundaries and exceptions?

## Consequences

What becomes easier, harder, required, or intentionally deferred?

## Validation and review

What evidence will validate or cause reconsideration of the decision?
```

## Initial records

- [ADR-0001: Grow architecture from end-to-end scenarios](0001-grow-architecture-from-scenarios.md)
- [ADR-0002: Use cREXX selectively](0002-use-crexx-selectively.md)
- [ADR-0003: Establish a portable monorepo skeleton](0003-establish-a-portable-monorepo-skeleton.md)
- [ADR-0004: Establish the framework security baseline](0004-establish-the-framework-security-baseline.md)
- [ADR-0005: Use JSON Schema for the common contract baseline](0005-use-json-schema-for-the-common-contract-baseline.md)
- [ADR-0006: Use a local journal for the M1 reference binding](0006-use-a-local-journal-for-the-m1-reference-binding.md)
- [ADR-0007: Separate local-synthetic and managed trust profiles](0007-separate-local-synthetic-and-managed-trust-profiles.md)
- [ADR-0008: Use Ed25519 and protected local state for the M2 reference trust binding](0008-use-ed25519-and-protected-local-state-for-the-m2-reference-trust-binding.md)
- [ADR-0009: Use a bounded in-process policy adapter for the M2 reference path](0009-use-a-bounded-in-process-policy-adapter-for-the-m2-reference-path.md)
- [ADR-0010: Use a locked local security journal and rebuild recovery for the M2 reference path](0010-use-a-locked-local-security-journal-and-rebuild-recovery-for-the-m2-reference-path.md)
- [ADR-0011: Establish the M3 scenario-control invariants](0011-establish-the-m3-scenario-control-invariants.md)
- [ADR-0012: Introduce a cost-controlled Google Cloud hosted preview during M3](0012-introduce-a-cost-controlled-google-cloud-hosted-preview-during-m3.md)

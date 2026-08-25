# ADR-0005: Use JSON Schema for the common contract baseline

Status: Proposed
Date: 2026-08-25

## Context

M1 needs transport-neutral, machine-validated contracts that can be consumed by
Rust and TypeScript without selecting an HTTP framework or event broker. The
representation must support strict objects, tagged variants, reusable
definitions, examples and negative conformance fixtures.

## Decision

Use UTF-8 JSON and JSON Schema Draft 2020-12 as the canonical M1 contract source.
Every schema has a stable identifier and explicit semantic version. A major
version may break existing valid messages; a minor version is additive and
backwards compatible for existing messages; a patch changes clarification or
validation evidence without changing the accepted instance set.

Validate schemas and fixtures with an independent TypeScript build-time tool.
Provide shared Rust and TypeScript types for implemented contracts, with tests
against the canonical examples. Do not infer transport, topic, route, storage
or authentication from the JSON representation.

The reference idempotency fingerprint uses deterministic serialization of the
typed M1 envelope. It is private delivery state, not a signature format or a
cross-implementation canonicalisation contract.

## Consequences

- Contracts remain inspectable and usable by current backend and frontend
  tooling.
- Schema/type drift becomes a checked failure rather than a documentation
  convention.
- Other encodings may be bound later but must demonstrate semantic equivalence.
- Signing or cross-language content fingerprints require a separate canonical
  representation decision; M1 does not silently claim RFC 8785 conformance.

## Validation and review

Founder review is required before acceptance. Validate the choice through all
six common schemas, positive and negative fixtures, Rust deserialisation,
TypeScript consumption and compatibility checks. Reconsider if a later
transport, streaming need, signature profile or non-JavaScript consumer cannot
preserve the semantics safely.

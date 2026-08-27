# Contract source

This directory is the implementation home for transport-neutral contract
schemas, examples and conformance fixtures. The logical definitions and their
authority remain under [`docs/architecture/logical/`](../docs/architecture/logical/README.md).

[`catalog.json`](catalog.json) lists all contract families. A family remains
`planned` until its detailed logical specification is reviewed. `C-001` to
`C-006`, `I-001` to `I-005` and `AZ-001` have accepted JSON Schema 2020-12
definitions, examples, negative fixtures and compatibility descriptors under
[`common/`](common/), [`identity/`](identity/) and
[`authorisation/`](authorisation/). Acceptance fixes their current semantics
and compatibility baseline; it does not claim that every consumer, transport
or deployment profile is implemented. In particular, `I-001` has a canonical
contract but no external provider binding in M2.

Contract source will be organised by family:

- `common/` for `C-001` to `C-006`;
- `demonstration/` for `D-001` to `D-004`;
- `presentation/` for `P-001` to `P-004`;
- `identity/` for `I-001` to `I-005`;
- `authorisation/` for `AZ-001` and later policy-decision bindings;
- `service/` for engagement, asset, knowledge, work, rule, AI, report and
  adapter contracts; and
- `operation/` for audit, analytics, health, recovery and delivery contracts.

`D-001` to `D-004` have accepted M3.1 logical specifications. They remain
schema-free and unimplemented; the catalogue's `agreed` status accepts their
semantics but is not a schema, runtime or deployment claim.

JSON Schema is the canonical representation; it does not select OpenAPI,
AsyncAPI, CloudEvents, Protobuf, an event broker or an API framework. Run
`pnpm check:contracts` to compile every registered schema and verify its
positive and negative fixtures and compatibility descriptor.

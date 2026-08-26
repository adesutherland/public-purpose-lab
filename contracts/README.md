# Contract source

This directory is the implementation home for transport-neutral contract
schemas, examples and conformance fixtures. The logical definitions and their
authority remain under [`docs/architecture/logical/`](../docs/architecture/logical/README.md).

[`catalog.json`](catalog.json) lists all contract families. A family remains
`planned` until its detailed logical specification is reviewed. `C-001` to
`C-006` have accepted JSON Schema 2020-12 definitions, examples, negative
fixtures and compatibility descriptors under [`common/`](common/). Acceptance
fixes the M1 semantics and compatibility baseline; it does not claim that every
consumer, transport or deployment profile is implemented.

Contract source will be organised by family:

- `common/` for `C-001` to `C-006`;
- `demonstration/` for `D-001` to `D-004`;
- `presentation/` for `P-001` to `P-004`;
- `identity/` for `I-001` to `I-005`;
- `authorisation/` for `AZ-001` and later policy-decision bindings;
- `service/` for engagement, asset, knowledge, work, rule, AI, report and
  adapter contracts; and
- `operation/` for audit, analytics, health, recovery and delivery contracts.

JSON Schema is the canonical M1 representation; it does not select OpenAPI,
AsyncAPI, CloudEvents, Protobuf, an event broker or an API framework. Run
`pnpm check:contracts` to compile every common schema and verify its registered
positive and negative fixtures and compatibility descriptor.

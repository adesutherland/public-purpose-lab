# Contract source

This directory is the implementation home for transport-neutral contract
schemas, examples and conformance fixtures. The logical definitions and their
authority remain under [`docs/architecture/logical/`](../docs/architecture/logical/README.md).

[`catalog.json`](catalog.json) lists all contract families. A family remains
`planned` until its detailed logical specification is reviewed. Schema files,
generated language bindings and transport mappings are added only after that
definition is sufficiently stable.

Contract source will be organised by family:

- `common/` for `C-001` to `C-006`;
- `demonstration/` for `D-001` to `D-004`;
- `presentation/` for `P-001` to `P-004`;
- `identity/` for `I-001` to `I-005`;
- `service/` for engagement, asset, knowledge, work, rule, AI, report and
  adapter contracts; and
- `operation/` for audit, analytics, health, recovery and delivery contracts.

The repository skeleton does not yet define payload schemas or select OpenAPI,
AsyncAPI, CloudEvents, Protobuf, JSON Schema, an event broker or an API
framework. Those are contract and ADR decisions, not build-system defaults.

# C-005: Component capability manifest

Status: Accepted

Version: 1.0.0

Owner: The component publishing its supported public capabilities

## Purpose

`C-005` publishes stable semantic capabilities, implemented contract versions,
supported deployment profiles and readiness dependencies. It lets a caller or
surface discover meaning without learning internal routes, topics, tables or
code structure.

## Manifest concepts

The manifest includes component identity, release, maturity, generation time,
supported profiles, semantic capabilities, contract/version ranges, readiness
dependencies and conformance evidence.

Maturity is explicit:

- `repository-skeleton` identifies a compilable boundary only;
- `in-development` identifies implemented behaviour with incomplete milestone
  or platform evidence; and
- `demonstrated` requires the stated scenario and profile evidence.

No M1 manifest may claim production readiness.

## Rules

- A capability identifier describes business or interaction intent, not a URL.
- A component lists only contract versions it actually checks and supports.
- Readiness dependencies name logical capabilities or contracts, not assumed
  network proximity.
- A manifest is descriptive. Possessing it does not authorise invocation.
- Removal or incompatible change follows `C-006` and cannot be hidden by a new
  deployment route.
- Evidence references identify the tests and profiles supporting the maturity
  claim.

Canonical source:
[JSON Schema](../../../../../contracts/common/c-005-component-capability-manifest.schema.json).
Conformance examples are listed in the
[fixture manifest](../../../../../contracts/common/fixtures.json).

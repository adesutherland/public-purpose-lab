# C-006: Contract compatibility descriptor

Status: Working baseline

Version: 1.0.0

Owner: `INT-01`, reviewed with producers and consumers of the described
contract

## Purpose

`C-006` states the exact schema, maturity, compatibility policy, examples,
deprecation and conformance evidence for one contract version. It prevents
implicit compatibility based on a shared transport or similar field names.

## Descriptor concepts

The descriptor identifies the contract family, name, semantic version, status,
schema identifier, compatibility mode, effective time, optional predecessor or
replacement versions, example references, producer/consumer expectations and
conformance evidence.

M1 uses these version rules:

- a major version may make an incompatible semantic or instance change;
- a minor version may add optional information or capability while preserving
  every instance valid under the earlier minor version;
- a patch may clarify documentation or evidence but does not change the set of
  valid instances; and
- a receiver supports only versions it explicitly declares and safely refuses
  all others.

An `agreed` schema is a reviewed semantic baseline. `implemented` additionally
requires an executable producer or consumer and named conformance evidence.
Neither status implies production or portfolio-wide adoption.

## Change and deprecation

Compatibility is assessed against both schema and meaning. An optional field
that changes authority, default behaviour or information disclosure can still
be semantically breaking. Deprecation states its replacement and review period;
old evidence remains interpretable.

Canonical source:
[JSON Schema](../../../../../contracts/common/c-006-contract-compatibility-descriptor.schema.json).
Conformance examples are listed in the
[fixture manifest](../../../../../contracts/common/fixtures.json).

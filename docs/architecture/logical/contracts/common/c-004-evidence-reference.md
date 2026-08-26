# C-004: Evidence reference

Status: Accepted

Version: 1.0.0

Owner: The component that retains the referenced evidence

## Purpose

`C-004` links a command, outcome, source, claim, transformation, rule, model
step, human decision or release to retained evidence without copying that
evidence into every interaction.

## Required concepts

An evidence reference identifies:

- the evidence record and kind;
- its owning component and opaque location/reference;
- creation time and information classification;
- optional media type, content digest and source version;
- retention and access-policy references; and
- optional predecessor evidence references for lineage.

The reference is not proof that the caller may read the evidence. The owner
authorises access, applies retention and explains absence or disposal.

## Integrity and privacy

- A digest states its algorithm and value. It supports integrity comparison but
  does not by itself prove authorship, time or lawful use.
- The reference contains no inline source text, prompt, model output,
  credential, token, grant or session material.
- Evidence location is an opaque logical reference rather than an internal file
  path, database key or publicly dereferenceable URL.
- Source, generated analysis, accepted finding, substantive evidence and
  released artifact remain distinct kinds and classifications.
- Lineage is append-oriented. Correction or retraction adds evidence and does
  not silently replace the original record.

## Failure and compatibility

An unavailable, unauthorised, disposed or incompatible evidence target is
reported explicitly by the evidence owner. A component must not treat a broken
reference as proof of the underlying claim.

Canonical source:
[JSON Schema](../../../../../contracts/common/c-004-evidence-reference.schema.json).
Conformance examples are listed in the
[fixture manifest](../../../../../contracts/common/fixtures.json).

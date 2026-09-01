# A-001 — Governed source intake

Status: Working draft implemented for the first Gate C transaction

Version: `0.1.0`

Owners: `UX-02` Workbench adapter and `CNT-01` source governance

## Purpose

`A-001` carries one authenticated source submission from the Workbench to the
source-governance component and returns the component-owned quarantine outcome.
It gives DS-03 a real business transaction without assigning validation,
staging, processing or legal authority to the intake step.

The first binding supports UTF-8 plain text and Markdown acquired by upload or
paste. A link can remain visible as a future acquisition mode, but this version
does not fetch remote content.

## Command

`source-intake.command` contains:

- the environment, demonstration session and bounded synthetic engagement;
- authenticated synthetic reviewer, role and application-session authority
  reference added by the backend, not trusted from the browser;
- explicit `governed-source-intake` purpose, correlation, causation and
  idempotency identifiers;
- acquisition mode, optional original filename, media type and exact byte
  count; and
- content plus required title, owner, rights, provenance and synthetic
  classification.

The command does not mean that the source is valid, staged, approved or
processed.

## Outcome and query

`source-intake.outcome` reports quarantined or refused status, a safe reason
code and, when quarantined, the immutable version identifier, digest and source
metadata. It never returns source content.

`source-intake.query` retrieves the previously recorded outcome by command ID
within the current environment. The gateway additionally checks that the
outcome belongs to the caller's bound demonstration session.

## Rules in the current binding

1. Only the established `synthetic-reviewer` with the
   `workbench-reviewer` role on `reviewer-workbench` may submit.
2. Only the approved synthetic engagement and purpose are accepted.
3. Classification must be `synthetic`; supported media types are `text/plain`
   and `text/markdown`; content must be non-empty and no larger than 64 KiB.
4. Title, owner, rights and provenance are required.
5. Upload mode requires an original filename.
6. An exact idempotent retry returns the recorded outcome; changed semantic
   input under the same key is refused.
7. A successful submission creates immutable version 1 in quarantine and
   emits metadata-only `source.received` and `source.quarantined` facts through
   a component outbox after durable broker acknowledgement.

## Failure and privacy behaviour

Unsupported authority, classification, media, size, content or metadata is
refused before a version is created. Component state failure is fail closed.
Source bodies remain in the `CNT-01` store and are excluded from HTTP outcomes,
operational events and logs.

This working contract is not yet agreed. Gate C validation, staging and
processing will extend or add contracts after their exact user actions and
component responsibilities are implemented and reviewed.

## Evidence

- canonical schema and fixtures: `contracts/source/`;
- component tests: `backend/components/cnt-01/src/lib.rs`; and
- end-to-end system check: `tools/smoke-m3-native.sh`.

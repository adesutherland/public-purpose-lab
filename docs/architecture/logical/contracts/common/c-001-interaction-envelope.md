# C-001: Interaction envelope

Status: Accepted

Version: 1.0.0

Owner: `INT-01`, with business meaning owned by the payload contract

## Purpose

`C-001` carries the routing, authority, time, correlation, classification and
security references needed to evaluate one public interaction without making a
transport part of its meaning. It can wrap commands, events, demonstration
control, trust commands, presentation cues, operational signals and analytical
projections.

M1 implements one local assurance command. The envelope does not authenticate
its contents and must not be exposed through an external transport until an
approved identity binding supplies that evidence.

## Participants and authority

The source component creates the envelope. `INT-01` validates contract and
delivery semantics. The target component validates the envelope and owns the
action decision. The requesting principal and any initiating actor are carried
through [`C-002`](c-002-authority-and-purpose-context.md); the transport may
narrow but never expand their authority.

## Required concepts

| Concept                                                 | Meaning                                                                         |
| ------------------------------------------------------- | ------------------------------------------------------------------------------- |
| `contractId`, `contractVersion`                         | Schema family and exact semantic version                                        |
| `messageId`, `messageType`, `messageKind`               | Unique interaction identity and semantic profile                                |
| `sourceComponent`, `targetComponent`, `audience`        | Claimed producer, receiver and intended consumer                                |
| `issuedAt`, optional `occurredAt`, optional `expiresAt` | RFC 3339 time basis; events state occurrence, expiring work states its deadline |
| `correlationId`, optional `causationId`                 | End-to-end transaction and immediate causal predecessor                         |
| `idempotencyKey`                                        | Required for command profiles and scoped to target, contract and operation      |
| `trace`                                                 | Non-authoritative diagnostic correlation                                        |
| `authority`                                             | `C-002` principal, purpose and constraint context                               |
| `classification`                                        | Information level, semantic categories and handling markers                     |
| `security`                                              | Opaque authentication/integrity references, never credential material           |
| `payload`                                               | Object governed by the named payload contract                                   |

## Invariants and failure

- A command without idempotency, authority, purpose, target, audience or
  classification is refused before business work.
- The receiver rejects wrong-environment, wrong-audience, expired,
  not-yet-valid or incompatible work.
- A repeated command is reconciled by idempotency; the key reused for different
  semantic content is an explicit conflict.
- `occurredAt` describes an event fact and is not supplied to make a command
  appear already accepted.
- Credentials, raw grants, cookies, private keys and session secrets are never
  payload or security metadata.
- Acceptance, refusal, expiry, duplicate and failure use
  [`C-003`](c-003-command-outcome-and-failure.md). Acceptance is not a business
  event.

Ordering is only guaranteed where the payload contract defines a narrow stream
or aggregate rule. Consumers otherwise tolerate repeated, delayed and
out-of-order delivery.

## Compatibility and evidence

The exact version uses the compatibility policy in
[`C-006`](c-006-contract-compatibility-descriptor.md). A consumer must refuse a
version it has not declared. Evidence includes schema validation, receiver
decision, outcome, correlation and a privacy-minimised delivery record.

Canonical source:
[JSON Schema](../../../../../contracts/common/c-001-interaction-envelope.schema.json).
Positive and negative examples are registered in the
[fixture manifest](../../../../../contracts/common/fixtures.json).

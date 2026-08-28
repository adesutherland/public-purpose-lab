# ADR-0023: Retain single-server JetStream for M3.4

Status: Accepted
Date: 2026-08-28

## Context

ADR-0013 requires review of the single-server NATS JetStream binding before
M3.4. M3.4 needs durable, acknowledged identity and presentation events, but it
does not claim high availability. Introducing a three-node broker solely for a
short-lived synthetic demonstration would materially increase local and hosted
cost and obscure the identity/resilience questions under test.

## Decision

Retain one file-backed NATS JetStream server for M3.4 local, Compose, Minikube
and ephemeral hosted evidence. Add separate least-privilege identity-broker,
Director and Presentation Gateway subjects and durable consumers. Continue to
use explicit acknowledgement, bounded redelivery, message/byte/age limits and
component-owned idempotency.

Broker restart is an M3.4 recovery case. Broker or volume loss is a safe
availability failure and may require a known reset; it is not a transparent
high-availability recovery claim. Readiness reports the limitation as
`single-server-development-assurance` wherever operational status is shown.

## Consequences

- M3.4 can test duplicate delivery and restart reconciliation with a small,
  consistent Kubernetes-compatible binding.
- A broker outage can interrupt the demonstration and no availability SLO is
  claimed.
- Shared hosted use remains short-lived, synthetic-only, supervised and
  explicitly in development.

## Validation and review

Evidence must cover broker restart with retained storage, duplicate delivery,
unavailable broker fail-closed readiness and safe recovery/reset. Select a
replicated topology before an availability commitment, unsupervised operation,
non-synthetic information or production-like qualification.

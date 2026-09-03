# ADR-0025: Bind Gate C quarantine to component-owned SQLite

Status: Accepted
Date: 2026-09-01
Accepted: 2026-09-03

## Context

DS-03 needs a real immutable source version, durable idempotency and a
metadata-only event outbox before the Workbench can truthfully show receipt and
quarantine. The first supported profiles are single-instance native,
containerised and Minikube demonstrations. Selecting a shared database or
general object store now would add an estate before validation, staging,
processing, retention, backup and hosted evidence requirements are known.

ADR-0018 already establishes component-owned SQLite for bounded,
single-instance M3 state. ADR-0024 requires this state-changing handler to
revisit whether the configurable component host still preserves ownership and
failure boundaries.

## Decision

Keep the shared `ppl-component-host` image, but give the `CNT-01` deployment a
dedicated source-governance module, SQLite database and persistent volume. No
other workload mounts or opens that database. The Presentation Gateway sends
the authenticated `A-001` command over its least-privilege NATS request/reply
channel; `CNT-01` owns validation, the transaction, idempotency, immutable
version record and lifecycle-event outbox.

Publish source lifecycle facts to a separate file-backed
`PPL_GATE_C_SOURCE` JetStream subject and mark the component outbox row
published only after the broker acknowledgement. The existing Gate A subject
continues to feed the live Operations projection, but its ephemeral readiness
semantics are not treated as the durable business-fact record.

Use WAL mode, full synchronous commits, foreign keys and an explicit schema
version. Store the small bounded source body only in `CNT-01`; return and emit
metadata only. A committed result is safe to query or retry after response
loss.

This decision qualifies only the first synthetic single-instance transaction.
It does not select long-term content storage, backup/restore, retention,
malware scanning, multi-replica processing or hosted non-synthetic operation.

## Consequences

- Gate C gains a concrete durable transaction while preserving logical and
  deployment ownership.
- Native, Compose and Minikube use the same code and contract; each environment
  has its own state.
- The source body is not distributed through events, the gateway database or
  the Operations projection.
- The bounded source event stream retains at most 1,000 messages, 4 MiB or
  seven days; it is implementation evidence, not the final audit record.
- SQLite and a single-writer deployment constrain scaling and recovery until a
  later binding is qualified.
- Backup and restore must preserve source/evidence state separately from trust
  material and application-session state.

## Validation and review

The founders accepted this decision on 3 September 2026 after the implementation
and walkthrough proved upload and paste, restart-safe exact retry,
changed-duplicate refusal, content exclusion from outcomes, events and logs,
and environment-bound status lookup.

Revisit before Gate C adds a separate or privileged content-scanning service,
or before multi-replica operation, backup and restore, hosted non-synthetic
data or retention requirements are selected. The bounded in-component text
validation required by DS-03 does not by itself change this storage decision.

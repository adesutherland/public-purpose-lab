# ADR-0018: Use component-owned SQLite state for the M3 single-instance runtime

Status: Accepted

Date: 2026-08-27

## Context

M3.3 needs restart-safe scenario lifecycle, registration, idempotency, cue,
outcome, checkpoint and outbox decisions. A successful broker acknowledgement
cannot substitute for a durable receiving-component decision, and an
interrupted process must reconcile rather than blindly repeat uncertain work.

The current milestone is deliberately single-instance and synthetic-only. It
does not need multi-writer scale, cross-region availability, a shared reporting
database or a general database platform. Selecting and operating a networked
database now would increase local footprint and hosted cost before a scenario
requires it.

The existing M1/M2 JSONL journals are bounded assurance records, not a suitable
transactional store for the related state, inbox and outbox changes required by
M3.

## Decision

Use SQLite for the M3.3 single-instance control stores, with one database owned
by `CTL-01` and a separate database owned by `CTL-02`.

Each component:

- owns its schema, migrations, connection and recovery decisions;
- accesses no table owned by the other component;
- exchanges only accepted contract identifiers and safe evidence references;
- records inbox identity, semantic fingerprint, authority/policy reference,
  state revision, outcome and any outbox event in one transaction;
- publishes an outbox item only after commit; and
- acknowledges a consumed JetStream message only after its inbox and semantic
  decision are durable.

Use write-ahead logging, foreign-key enforcement, a bounded busy timeout and a
durability setting appropriate to conclusive control decisions. Pin and test
the exact SQLite library and settings during implementation. Startup checks the
schema version, database accessibility and integrity and reconciles pending
outbox/inbox/uncertain operations before readiness.

Run one writer process and one replica for each database. Local containers use
separate named volumes. Minikube uses one block-backed `ReadWriteOnce`
PersistentVolumeClaim for each component. The databases must not be placed on a
shared network filesystem or mounted by concurrent replicas.

NATS JetStream owns its own separate file storage. It cannot query either
component database, and neither SQLite database becomes an integration bus.

Reset marks or supersedes disposable session state through component-owned
semantic operations. It preserves security, idempotency, operation and evidence
history. Corruption or migration ambiguity fails readiness closed; the runtime
does not delete or recreate the store automatically.

The private M3.3 hosted health/contract smoke may use activation-scoped storage
only. Evidence required after teardown is exported through the protected
lifecycle evidence path before the volume is destroyed. M3.3 does not qualify
backup, restore, long-term evidence custody or recovery of real data.

## Alternatives considered

- **Extend the JSONL journals:** does not provide the multi-record transaction,
  indexed reconciliation and schema evolution needed for M3 control state.
- **PostgreSQL:** a likely later choice where concurrent writers, larger data
  ownership or operational support justifies it, but it adds a service,
  credentials, backup and cost surface prematurely.
- **One shared SQLite database:** could make local transactions convenient but
  would let co-deployment silently merge `CTL-01` and `CTL-02` ownership.
- **JetStream key-value/object storage as the database:** couples component
  authority and state recovery to the transport and weakens replaceability.
- **In-memory state:** cannot supply restart, idempotency or uncertain-operation
  evidence.

## Consequences

- Local and Minikube operation remain small while durable component decisions
  become testable.
- Separate files and repositories preserve logical ownership even though both
  components use the same database product.
- Transactional inbox/outbox handling makes at-least-once transport explicit
  and supports deterministic redelivery.
- One-replica and block-volume constraints must be enforced by configuration,
  readiness and deployment tests. Horizontal scale is unsupported.
- SQLite corruption, migration and filesystem behaviour become implementation
  risks requiring safe failure and recovery evidence.
- Moving a component to PostgreSQL or another store remains possible behind its
  repository interface but requires migration and parity evidence.
- Backup/restore and authoritative long-term audit retention remain later
  decisions rather than implied SQLite capabilities.

## Validation and review

Evidence must demonstrate:

- atomic state/outcome/outbox commit and rollback under injected failure;
- identical duplicate return and changed-content conflict refusal across
  restart;
- lost broker acknowledgement followed by redelivery without duplicate
  semantic effect;
- startup reconciliation of pending outbox and uncertain operations;
- schema migration, version mismatch, unavailable storage and corrupt-store
  fail-closed behaviour;
- separate files, permissions, volume claims and effective process access;
- enforced single replica and refusal of unsafe shared/concurrent storage; and
- teardown that exports required safe evidence and leaves no activation volume.

Review the choice before a component needs multiple replicas or writers,
shared/long-lived hosted state, real data, formal backup/restore or materially
larger query and reporting workloads.

## Reference material

- [SQLite write-ahead logging](https://www.sqlite.org/wal.html)
- [Kubernetes persistent volumes](https://kubernetes.io/docs/concepts/storage/persistent-volumes/)
- [Kubernetes single-instance stateful application](https://kubernetes.io/docs/tasks/run-application/run-single-instance-stateful-application/)

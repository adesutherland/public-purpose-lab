# ADR-0006: Use a local journal for the M1 reference binding

Status: Accepted
Date: 2026-08-25
Accepted: 2026-08-26

## Context

M1 must demonstrate acceptance, refusal, expiry, duplicate handling, restart
and privacy-minimised evidence before the project has selected an event broker,
API framework or operational database. An in-memory example would conceal the
restart and idempotency boundary; a platform selection would be premature.

## Decision

Implement one local command-file/JSON-outcome adapter in the existing Rust
framework host. Use an explicitly configured append-only JSON-lines journal for
the minimum delivery, idempotency and audit evidence needed by the slice.

Hold an exclusive cross-platform file lock across read, decision and append.
Store irreversible idempotency, command and principal digests rather than the
command payload or raw authority context. A partial or invalid journal makes
interaction readiness false and is never repaired silently.

Run the same adapter natively and in the existing non-root container. Keep
`INT-01` in the framework-host deployable; this decision does not create a
separate service or select the future broker, database or audit store.

## Consequences

- Duplicate and restart behaviour becomes executable now.
- The physical journal co-locates delivery state and audit evidence while their
  logical ownership stays distinct.
- The binding is qualified only for one host and one writable state directory.
- Multiple replicas, network filesystems, high availability, tamper evidence,
  backup and long-term evidence retention remain unqualified.

## Validation and review

The founders accepted this M1 binding on 26 August 2026. Safe first delivery,
duplicate delivery, conflicting reuse, concurrency, restart, expiry,
corruption, unavailable state and disclosure passed the native suite. Hosted
CI also passed the Linux build and two-container restart/idempotency check over
one shared volume.

Replace or extend this ADR before adding an external listener, event broker,
multi-replica deployment or authoritative audit retention.

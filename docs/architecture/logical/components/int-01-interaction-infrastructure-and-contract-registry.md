# INT-01: Interaction infrastructure and contract registry

Status: Working baseline; M1 implementation in development

Last reviewed: 25 August 2026

## Purpose

`INT-01` owns transport-neutral contract publication, compatibility evidence,
message carriage semantics, delivery coordination and the minimum durable state
needed to make repeated delivery safe. It does not own business meaning,
identity, domain decisions or the evidence content referenced by an
interaction.

The M1 implementation is a local assurance adapter in the existing framework
host. It is a revisable baseline, not the selection of the eventual API, event
broker, database or audit platform.

## Accountable ownership

Public Purpose Lab owns the applied contract suite and M1 evidence. Producers
own correct construction of their messages. Receivers own validation,
authorisation and action decisions. `INT-01` owns:

- publication of `C-001` to `C-006` schemas, examples and compatibility state;
- carriage-neutral rules for correlation, causation, expiry and idempotency;
- compatibility refusal before unsupported work reaches a component;
- delivery/reconciliation state needed to prevent duplicate application;
- safe command outcomes for interaction-level decisions;
- privacy-minimised delivery evidence and readiness diagnostics; and
- conformance fixtures reusable by later transports.

## Non-responsibilities

`INT-01` does not:

- authenticate an actor or workload or make metadata trustworthy;
- decide domain, professional, legal, regulatory or report authority;
- turn an accepted command outcome into a business event;
- expose a browser directly to an event broker or component-private endpoint;
- retain general payloads, source content, credentials, keys, grants or
  sessions as delivery evidence;
- define global ordering where a domain contract has not done so;
- provide analytical or operational truth by replaying transport records; or
- require a logical component to be a separate process, service or topic.

## Accepted interactions

The logical component accepts versioned envelopes for contract validation,
compatibility assessment, delivery and reconciliation. A transport binding
must supply an authenticated workload context before it is used outside the M1
local assurance profile.

M1 implements one semantic capability:

`interaction.conformance-probe`

It accepts a synthetic, expiring demonstration-control envelope targeted to
`INT-01`, validates the current common contract rules and durably records one
privacy-minimised operation. This capability exists only to prove the spine. It
does not mutate a business component or establish identity.

## Produced outcomes and evidence

The component produces `C-003` outcomes with `C-004` references for accepted,
refused, expired, duplicate or failed delivery. It publishes a `C-005` manifest
and the `C-006` descriptors for the six common contracts.

The M1 journal record contains:

- record and outcome identifiers and time;
- message, correlation, contract and environment references;
- irreversible command, idempotency and principal digests;
- principal type, safe decision code and whether the reference operation was
  applied; and
- a `C-003` outcome with an opaque `C-004` journal evidence reference.

It does not contain the envelope, payload, raw authority context, idempotency
key or authentication metadata.

## Idempotency and ordering

Idempotency is scoped to target component, contract family/version and semantic
operation. The M1 adapter binds an idempotency digest to a deterministic typed
command fingerprint while holding an exclusive state lock.

- First valid delivery records and applies the reference operation once.
- Same key and same fingerprint returns `duplicate` and identifies the original
  outcome without applying again.
- Same key and different fingerprint returns `idempotency_conflict`.
- Restart rebuilds reconciliation state from the append journal.
- Corrupt or incomplete history makes readiness false; history is not silently
  discarded or rewritten.

No global event ordering is promised. Later contract families define any
aggregate-specific sequence or expected-version rule.

## Security and privacy boundary

The component follows the
[framework security model](../../security/framework-security-model.md) and
[M1 threat model](../../security/m1-threat-model.md). It validates environment,
principal type, actor separation, purpose, target, audience, classification,
time and prohibited sensitive field names.

M1's local invocation boundary is not authentication. The adapter has no
external listener and must not be placed behind one. `IAM-01` and a reviewed
transport ADR must establish the requesting workload and any actor before later
bindings reuse the capability.

## Failure and recovery

Input exceeding the bounded size or failing JSON/type validation is rejected
without echoing content. A decision is not accepted unless its journal record
is appended and synchronized. Storage, lock, parse or synchronization failure
returns a safe error and makes readiness false where continuity is uncertain.

The reference journal is qualified for one host and one writable state
directory only. It is not a high-availability, tamper-evident or long-term
audit store. A later store must migrate or deliberately reset delivery state
without making uncertain commands eligible for duplicate application.

## Observability

Health reports distinguish:

- process/software health;
- interaction-state readiness;
- configured environment and implementation maturity; and
- safe journal record counts or a bounded failure code.

They never emit input content, authority assertions, credential material,
filesystem contents or raw principal/idempotency identifiers. Business
completion is outside this health contract.

## Deployment and replaceability

The M1 Rust package is linked into `ppl-framework-host`; it is not a separate
service. Native invocation and the non-root container use the same types,
fixtures and journal behaviour. The container has a read-only root filesystem
and one declared writable state mount.

A future HTTP, broker or database binding must preserve schema/version checks,
authenticated workload context, receiver authority, idempotency conflict,
outcome, evidence, corruption and disclosure semantics. Product access or
transport delivery is never accepted as authority.

## Conformance evidence

M1 evidence must demonstrate:

1. all common schemas compile and every registered positive/negative fixture
   has the expected result;
2. Rust and TypeScript consumption of the canonical examples;
3. one valid command accepted exactly once;
4. sequential, concurrent and post-restart duplicates apply no second
   operation;
5. conflicting idempotency reuse is refused;
6. unsupported version, wrong environment/target/audience, invalid authority,
   expiry and restricted fields are safely refused;
7. the journal and diagnostics contain no payload or reusable security value;
8. corrupt history makes readiness false; and
9. native and container-hosted forms run the same conformance command.

## Limitations and next review

M1 remains `in-development` until the framework security model, threat model,
ADRs and exit evidence receive founder review. This specification does not
qualify external transport, authentication, distributed delivery, hosted
persistence or production operation. Review it before M2 identity exposure and
again when a broker, API or durable operational store is proposed.

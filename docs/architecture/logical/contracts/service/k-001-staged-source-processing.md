# K-001 - Staged source processing

Status: Implemented Gate C processing baseline; final subject to review sign-off by exception

Version: `0.1.0`

Owners: `KNO-01` knowledge processing and `CNT-01` source governance; `UX-02` Workbench adapter

## Purpose

`K-001` completes the deliberately small DS-03 processing path. `KNO-01`
durably consumes the metadata-only `source.staged` fact for one exact immutable
source version, records idempotent acceptance, emits a visible lifecycle and
performs only basic text inspection. It does not implement or claim RAG
ingestion, retrieval, semantic understanding, evidence quality or compliance.

## Staged-source consumption

The existing file-backed `PPL_GATE_C_SOURCE` JetStream stream carries the
versioned, content-free `source.staged` fact. `KNO-01` binds a durable explicit-
acknowledgement consumer. The source-version identifier and originating event
identifier form the retained receipt boundary; exact redelivery returns the
same processing record and cannot create a second terminal result.

Only the authenticated `KNO-01` workload may request the exact staged body from
`CNT-01` over `ppl.gate-c.processing-input.CNT-01`. `CNT-01` verifies the
environment, Demonstration Session, source version, staged lifecycle, requester
component and `bounded-source-processing` purpose before returning protected
content. The response is point-to-point component traffic and is not published
to JetStream, Operations or either browser surface.

## Lifecycle and bounded result

The component-owned SQLite record contains:

1. `accepted`, representing idempotent receipt and queued work;
2. `processing`, paired with `processing.started`; and
3. exactly one terminal `completed` or `failed` state.

Completed processing verifies the retained SHA-256 digest and exact byte count,
then records byte, line and section counts plus a maximum 240-character safe
preview. Markdown headings define sections when present; otherwise separated
non-empty text blocks define sections. These deterministic operations do not
interpret meaning or establish authority.

`[[PPL_PROCESSING_FAILURE]]` is an allow-listed synthetic failure fixture. It
creates a visible `processing.failed` result with safe reason
`processing-fixture-failure`; it is not a content-classification rule.

## Views and disclosure

- Workbench returns lifecycle stages, identifiers, counts and the bounded safe
  preview to the established synthetic reviewer.
- Presentation returns lifecycle, component, counts and limitation only. It has
  no business-operation control and never receives the source body or preview.
- Operations returns correlated `CNT-01` and `KNO-01` events, timestamps,
  refusals and safe event/source-version references. General logs contain safe
  reason codes only.

## Restart and failure behaviour

The `KNO-01` deployment has a separate persistent volume and SQLite store.
Startup reconciles accepted or interrupted work; completed or failed records are
conclusive and are not re-executed. The transactional outbox retains one event
identifier for each lifecycle transition until both durable and operational
publication succeed.

Malformed, empty, oversized, unvalidated or staging-refused sources produce no
`source.staged` fact and therefore never enter `KNO-01`. Missing or mismatched
protected input fails closed. Distributed multi-writer operation, backup and
restore, RAG ingestion and non-synthetic data remain outside this Gate C
baseline.

## Evidence

- canonical schema and fixtures: `contracts/knowledge/`;
- component tests: `backend/components/kno-01/src/lib.rs`;
- source-boundary tests: `backend/components/cnt-01/src/lib.rs`; and
- end-to-end and restart check: `tools/smoke-m3-native.sh` through the retained
  Minikube profile.

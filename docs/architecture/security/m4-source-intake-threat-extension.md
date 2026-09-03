# Gate C source-intake threat extension

Status: Gate C closure candidate covering complete DS-03 processing

Date: 2026-09-01

This extension applies the accepted framework and M3 security models to the
first operation that carries untrusted source content. It covers receipt,
quarantine, bounded validation, reviewer-controlled staging and the deliberately
basic `KNO-01` processing lifecycle.

## Protected assets and boundaries

- Source content and metadata cross from the synthetic reviewer's browser,
  through the Presentation Gateway, to `CNT-01` over authenticated NATS.
- External-human application state and synthetic reviewer binding remain owned
  by the Presentation Gateway and `IAM-01`; the browser cannot assert actor,
  authority, environment, session or purpose fields.
- `CNT-01` alone owns the source body, immutable version record, idempotency
  result and lifecycle-event outbox.
- Only the authenticated `KNO-01` workload can request the exact staged body
  over the protected point-to-point processing-input subject. It owns a
  separate idempotent processing record, derived counts, bounded preview and
  lifecycle outbox.
- Operations receives metadata-only facts. It is not a source-content store.
- Workbench may return the bounded preview to its established synthetic
  reviewer. Presentation receives processing metadata and counts only and has
  no business-operation control.

## Threats and current controls

| Threat                                                                    | Current control                                                                                                                                                       | Remaining limit                                                                                              |
| ------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| Real or confidential data submitted to a synthetic environment            | Visible confirmation, contract classification fixed to `synthetic`, component refusal of any other classification                                                     | A typed confirmation cannot prove the content is synthetic; demonstrations must use controlled fixtures only |
| Browser forges synthetic actor, role, environment, session or authority   | Backend derives those fields from the protected application session and requires the bound `synthetic-reviewer`, role and Workbench surface                           | Compromised authorised browser session remains within the application-session threat model                   |
| Oversized or unsupported content consumes resources                       | Browser previews exact byte size; `CNT-01` enforces 64 KiB, two media types, non-empty content and no NUL byte                                                        | Full hostile-content and decompression/archive analysis is deferred because those formats are not admitted   |
| Source content leaks through events, status or logs                       | Outcome and operational event types contain metadata and digest only; component logs safe reason codes                                                                | Database and backup access control is not yet qualified                                                      |
| Retry creates duplicate source versions                                   | Durable semantic fingerprint and idempotency record return the first outcome; changed content under the same key is refused                                           | Distributed multi-writer idempotency is not qualified                                                        |
| Gateway response is lost after commit                                     | Outcome can be queried by command ID or the exact operation retried                                                                                                   | Query transport remains bounded to the current single-instance event profile                                 |
| Another environment or scenario reads an outcome                          | Component query is environment-bound; Gateway compares the returned demonstration session to the caller's synthetic session                                           | Broader engagement-level authorisation will be required as more than one engagement is implemented           |
| Quarantined text is mistaken for validated, staged or processed knowledge | Separate lifecycle states; `KNO-01` consumes only durable `source.staged`; every UI states that basic processing is not RAG ingestion or understanding                | Later retrieval and evidence-quality claims require Gate D qualification                                     |
| Deterministic text checks are mistaken for malware or truth assurance     | UI and contract list the exact bounded checks and explicitly disclaim malware, truth, authority and general content-safety qualification                              | Later formats or non-synthetic inputs require a separately selected scanning and governance binding          |
| Browser or gateway stages a source without reviewer authority             | Gateway derives actor/session fields; `CNT-01` requires successful validation and an exact-resource `AZ-001` permit from deployed `AUT-01`                            | The first policy uses synthetic scenario assertions and is not an external organisational authority          |
| Policy service is unavailable or returns an incomplete decision           | `CNT-01` treats deny, not-applicable, indeterminate, malformed and unavailable decisions as refusal                                                                   | Multi-policy reconciliation and external policy-administration availability are not qualified                |
| Staged event is duplicated or response is lost                            | Component-owned idempotency and transactional outbox retain one conclusive decision and metadata-only event                                                           | Distributed multi-writer staging is not qualified                                                            |
| Database loss or trust-domain recovery silently changes evidence          | Source database is separate from trust and application-session state; no recovery claim is made                                                                       | Backup, restore, retention and evidence reconciliation remain an M6 qualification item                       |
| Source body leaks through the processing path                             | Body travels only on the NKey-restricted point-to-point `CNT-01` to `KNO-01` subject; events, logs and Presentation contain identifiers, counts and safe reasons only | KNO-01 retains a bounded derived preview; database and backup access remain unqualified                      |
| Duplicate staged facts create repeated terminal results                   | Durable KNO-01 receipt is unique by source version/event; component-owned SQLite state and a transactional outbox permit one terminal transition                      | Distributed multi-writer and cross-region failover are not qualified                                         |
| Restart loses or silently repeats accepted work                           | Persistent KNO-01 state reconciles accepted/processing work; completed/failed records are conclusive and not re-executed                                              | Backup/restore and destructive storage recovery remain M6 work                                               |
| Processing result is mistaken for knowledge or assurance                  | The implementation exposes only digest verification, bytes, lines, sections and a bounded preview with explicit UI/contract limitations                               | RAG ingestion, retrieval, semantic interpretation and evidence quality remain Gate D work                    |

## Required adverse evidence

The executable checks must show non-synthetic, empty, oversized and changed
idempotent submissions being refused; restart-safe exact redelivery; absence of
source body from outcome and event JSON; environment-bound lookup;
malformed/hostile validation refusal; staging-authority refusal; successful and
failed processing; absence of source body from events, logs and Presentation;
and processing restart/reconciliation without a second terminal result.

# Gate C source-intake threat extension

Status: Working draft extended through DS-03 validation and staging

Date: 2026-09-01

This extension applies the accepted framework and M3 security models to the
first operation that carries untrusted source content. It now covers receipt,
quarantine, bounded validation and reviewer-controlled staging. Processing
requires further review as its executable slice is added.

## Protected assets and boundaries

- Source content and metadata cross from the synthetic reviewer's browser,
  through the Presentation Gateway, to `CNT-01` over authenticated NATS.
- External-human application state and synthetic reviewer binding remain owned
  by the Presentation Gateway and `IAM-01`; the browser cannot assert actor,
  authority, environment, session or purpose fields.
- `CNT-01` alone owns the source body, immutable version record, idempotency
  result and lifecycle-event outbox.
- Operations receives metadata-only facts. It is not a source-content store.

## Threats and current controls

| Threat                                                                    | Current control                                                                                                                             | Remaining limit                                                                                              |
| ------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| Real or confidential data submitted to a synthetic environment            | Visible confirmation, contract classification fixed to `synthetic`, component refusal of any other classification                           | A typed confirmation cannot prove the content is synthetic; demonstrations must use controlled fixtures only |
| Browser forges synthetic actor, role, environment, session or authority   | Backend derives those fields from the protected application session and requires the bound `synthetic-reviewer`, role and Workbench surface | Compromised authorised browser session remains within the application-session threat model                   |
| Oversized or unsupported content consumes resources                       | Browser previews exact byte size; `CNT-01` enforces 64 KiB, two media types, non-empty content and no NUL byte                              | Full hostile-content and decompression/archive analysis is deferred because those formats are not admitted   |
| Source content leaks through events, status or logs                       | Outcome and operational event types contain metadata and digest only; component logs safe reason codes                                      | Database and backup access control is not yet qualified                                                      |
| Retry creates duplicate source versions                                   | Durable semantic fingerprint and idempotency record return the first outcome; changed content under the same key is refused                 | Distributed multi-writer idempotency is not qualified                                                        |
| Gateway response is lost after commit                                     | Outcome can be queried by command ID or the exact operation retried                                                                         | Query transport remains bounded to the current single-instance event profile                                 |
| Another environment or scenario reads an outcome                          | Component query is environment-bound; Gateway compares the returned demonstration session to the caller's synthetic session                 | Broader engagement-level authorisation will be required as more than one engagement is implemented           |
| Quarantined text is mistaken for validated, staged or processed knowledge | Separate `quarantined` state, explicit UI limitation and no `KNO-01` event or call in this slice                                            | Gate C cannot close until later states and refusals are implemented and evidenced                            |
| Deterministic text checks are mistaken for malware or truth assurance     | UI and contract list the exact bounded checks and explicitly disclaim malware, truth, authority and general content-safety qualification    | Later formats or non-synthetic inputs require a separately selected scanning and governance binding          |
| Browser or gateway stages a source without reviewer authority             | Gateway derives actor/session fields; `CNT-01` requires successful validation and an exact-resource `AZ-001` permit from deployed `AUT-01`  | The first policy uses synthetic scenario assertions and is not an external organisational authority          |
| Policy service is unavailable or returns an incomplete decision           | `CNT-01` treats deny, not-applicable, indeterminate, malformed and unavailable decisions as refusal                                         | Multi-policy reconciliation and external policy-administration availability are not qualified                |
| Staged event is duplicated or response is lost                            | Component-owned idempotency and transactional outbox retain one conclusive decision and metadata-only event                                 | Distributed multi-writer staging is not qualified                                                            |
| Database loss or trust-domain recovery silently changes evidence          | Source database is separate from trust and application-session state; no recovery claim is made                                             | Backup, restore, retention and evidence reconciliation remain an M6 qualification item                       |

## Required adverse evidence

The executable checks must show non-synthetic, empty, oversized and changed
idempotent submissions being refused; restart-safe exact redelivery; absence of
source body from outcome and event JSON; environment-bound lookup;
malformed/hostile validation refusal; and staging-authority refusal. Before
Gate C closes, the walkthrough must add processing restart/reconciliation
evidence.

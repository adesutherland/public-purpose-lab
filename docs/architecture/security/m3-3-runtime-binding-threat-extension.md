# M3.3 runtime-binding threat extension

Status: Accepted M3.3 runtime-binding threat baseline; bounded implementation evidence completed

Version: 0.2.0

Last reviewed: 28 August 2026

Extends:
[M3 Scenario Director threat model](m3-threat-model.md) and
[M3.2 presentation and hosted-binding threat extension](m3-2-presentation-threat-extension.md)

Related accepted decisions: ADR-0017 to ADR-0020

## Scope and claim boundary

This extension covers the accepted M3.3 physical bindings for canonical
scenario packages, component-owned SQLite state, transactional inbox/outbox
delivery, operational/logical time separation, bounded reset/fault adapters,
one application image in two process modes and the local development-assurance
browser session.

It also covers the deliberately narrow private Google Cloud application smoke:
the same image is deployed without ingress and must fail interactive readiness
while managed trust is absent.

The linked evidence record closes these controls only for the bounded M3.3
profiles. It does not qualify managed trust, Google presenter login, hosted
workload identity, synthetic application sessions, shared access, high
availability, backup and restore, real data or production security. Those
remain M3.4 or later gates.

## New and refined trust boundaries

| Boundary                            | Trust position                                                       | Required M3.3 property                                                                                               |
| ----------------------------------- | -------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| Repository package source to build  | Reviewed first-party synthetic content, not runtime authority        | Closed schemas, exact source revision, canonical digest, prohibited-content checks and no mutable fetch/upload path. |
| Application image to runtime mode   | Shared immutable code artifact with distinct operational authority   | Exact mode, separate state/NATS material, least privilege and mode-confusion refusal.                                |
| `CTL-01` SQLite store               | Authoritative only for Director control decisions                    | Single writer, transactional inbox/state/outbox, migration/integrity checks and fail-closed recovery.                |
| `CTL-02` SQLite store               | Authoritative only for registry/cue/outcome decisions                | Separate file and access, current generation/revision, transactional outcomes and no cross-store query.              |
| SQLite/PersistentVolume attachment  | Durable single-instance state, not a concurrent database service     | One replica, block-backed `ReadWriteOnce`, no shared filesystem and no automatic destructive recovery.               |
| Component process to NATS           | At-least-once transport with workload-specific permissions           | TLS/NKey, separate subjects/identities, explicit acknowledgement after durable decision and bounded storage.         |
| Local development-assurance session | Synthetic test identity, not external authentication                 | Exact local profile, loopback/port-forward access, short backend session, visible banner and hosted rejection.       |
| Scenario logical clock              | Declared synthetic test data                                         | Manual forward-only revisions and no influence over operational/security validity.                                   |
| Reset and fault adapters            | Elevated but component-owned assurance capability                    | Named closed operation, exact session/target, expiry, retained evidence and no generic administration.               |
| Private hosted application smoke    | Untrusted as an interactive environment until managed controls exist | No ingress, development adapter, synthetic root or interactive readiness; exact image and conclusive teardown.       |

The common image does not make its two modes one principal. Conversely,
separate processes do not make shared code intrinsically safe; dependencies and
build provenance remain common risks.

## Threat register extension

| ID        | Threat                                                                                         | Required M3.3 control                                                                                                                                   | Evidence gate or residual risk                                                                                                |
| --------- | ---------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| `T-M3-46` | Different parsers or serialisers calculate different package meaning/digests                   | I-JSON input, duplicate-key refusal, JSON Schema 2020-12, RFC 8785 canonicalisation, SHA-256 and shared official/cross-runtime vectors                  | Library implementation and numeric/Unicode edge cases require conformance evidence.                                           |
| `T-M3-47` | A package or fixture is substituted between review, build and admission                        | Closed manifest, per-file digest, source revision, immutable image digest and admission record; no runtime fetch/upload                                 | Repository/build authority is sufficient only for the first reviewed package; external publishers require signing governance. |
| `T-M3-48` | Malicious package content triggers code, route access, resource exhaustion or ambiguous fields | No runtime extraction/execution; unknown-field, URL/route/subject/credential/executable and bounded-size/count refusal                                  | Prohibited-content heuristics supplement, but do not replace, closed schemas and review.                                      |
| `T-M3-49` | SQLite partial writes or process interruption invent, lose or repeat a control decision        | One transaction for inbox/state/outcome/outbox, durability settings, expected revision and startup reconciliation                                       | Filesystem and SQLite bindings require crash/fault evidence; this is not HA.                                                  |
| `T-M3-50` | Two replicas or shared mounts produce split-brain state or lock failure                        | One replica, one writer, block-backed `ReadWriteOnce`, deployment admission/readiness checks and no shared network filesystem                           | Accidental scale/config drift remains an operational risk and must fail before traffic.                                       |
| `T-M3-51` | One component reads or changes the other's state because both use SQLite                       | Separate files, mounts, process identities and repository modules; no cross-component foreign keys or queries                                           | Local host/operator access remains administrative capability and needs ordinary file protection.                              |
| `T-M3-52` | NATS acknowledgement is mistaken for a durable semantic decision                               | Transactional inbox/outbox, explicit acknowledgement after commit, immutable idempotency and uncertain reconciliation                                   | Lost acknowledgements and broker/process restart require end-to-end tests.                                                    |
| `T-M3-53` | One image mode gains the other mode's state or broker authority                                | Exact startup mode, separate configuration/secrets/volumes/NKeys and negative subject permission tests                                                  | Shared binary/dependencies remain a common compromise surface; review before higher assurance.                                |
| `T-M3-54` | The local development-assurance presenter path is enabled in hosted/shared use                 | Exact environment/trust/profile checks, loopback/port-forward binding, route/readiness refusal, prominent banner and configuration negative tests       | Build contains shared code; profile-downgrade and configuration injection remain priority tests.                              |
| `T-M3-55` | Logical scenario time extends a cue, registration, message, policy, credential or session      | Separate clock types/interfaces and persistence fields; operational expiry only; forward-only bounded logical operations                                | Code review and mutation tests must catch accidental clock-source reuse.                                                      |
| `T-M3-56` | Reset erases evidence/security state or creates a successor after partial failure              | Two named owner adapters, stop-before-reset, retained history, conclusive required outcomes and distinct successor ID                                   | Later business/application reset owners need independent controls and evidence.                                               |
| `T-M3-57` | Fault control becomes an arbitrary delay/admin interface or remains active invisibly           | One named one-shot profile, closed duration, session/slot binding, operational expiry, explicit clear and hosted absence                                | More fault profiles require threat review; no platform fault authority exists in M3.3.                                        |
| `T-M3-58` | Static assets, SSE cursors or diagnostics expose routes, subjects, sessions or credentials     | Same-origin backend, opaque non-authority cursor, CSP, origin/CSRF controls, safe diagnostics and disclosure scanning                                   | Full production browser-session assurance remains M3.4/M6 work.                                                               |
| `T-M3-59` | A hosted image is exposed or considered ready without managed trust                            | No Ingress/LoadBalancer, hosted profile declaration, managed-trust readiness dependency, development-adapter refusal and independent endpoint inventory | M3.3 hosted result is negative readiness evidence, not functional parity.                                                     |
| `T-M3-60` | Hosted teardown destroys state before evidence or leaves a volume/workload charging            | Activation-scoped resources, protected evidence export before off, exact workload/volume/endpoint inventory and conclusive teardown                     | Long-term evidence custody and backup/restore remain unqualified.                                                             |

## Required assurance and misuse cases

M3.3 implementation evidence must add:

- canonical package equality across supported environments and disagreement
  refusal for duplicate keys, numeric/Unicode edges and changed fixture content;
- package attempts containing a URL, NATS subject, credential-like field,
  executable content, unknown property, link, path traversal and over-limit
  content;
- process termination before commit, after state commit, before publish and
  before/after broker acknowledgement;
- component restart with pending outbox, duplicate inbox, conflicting content
  and uncertain cue outcome;
- attempted second replica, concurrent database attachment, corrupt database,
  wrong schema version and unwritable volume;
- Director credentials attempting Presentation subjects and the inverse;
- wrong process mode, another mode's state path and another mode's NKey;
- local development-adapter access under local, Minikube, hosted, managed and
  deliberately mismatched environment declarations;
- logical-time movement alongside operational cue/session expiry;
- reset from a non-terminal state, partial target failure, duplicate reset and
  prior-session event after successor creation;
- fault activation with excessive duration, wrong session/slot, duplicate,
  expiry, clear and attempted hosted use;
- copied SSE cursor, wrong session/generation and forged/late outcome;
- browser bundle, storage, network, event, database, log and evidence disclosure
  scans; and
- hosted deployment with no endpoint, development adapter or managed readiness,
  followed by exact workload/volume/network/task teardown evidence.

## Binding acceptance gates

The founders accepted this extension on 27 August 2026. Implementation may
proceed only with these controls represented in the work plan and tests:

1. canonical package parsing/digest code has independent vectors and refuses
   ambiguity before admission;
2. component state is transactional, separate and unable to start under an
   unsupported replica/storage profile;
3. NATS redelivery cannot duplicate a semantic effect or bypass receiver
   authority;
4. local development identity is visibly synthetic and technically unavailable
   in hosted/shared profiles;
5. scenario time, reset and faults remain typed component controls and cannot
   reach protected clocks, arbitrary storage or platform administration;
6. browser delivery is same-origin/backend-mediated and free of broker/security
   material; and
7. hosted M3.3 is ingress-free, expected-not-ready for interactive use and
   conclusively torn down.

Implementation evidence may close individual controls but cannot promote M3.3
to shared or production use. M3.4 must extend this model for managed root and
issuer custody, Google OIDC, workload identity, protected application sessions,
reconnect/restart cases and a functional hosted event/presentation path.

## Residual risk and review point

The largest residual risks after the M3.3 evidence cycle are
implementation-specific:
canonicalisation library correctness, database/filesystem crash behaviour,
profile downgrade, shared-image dependency compromise, browser session
construction, NATS TLS/NKey generation and hosted managed identity. The bounded
M3.3 cases are evidenced; broader and higher-assurance cases remain open for
M3.4 or later.

This extension was reviewed after the schema/package tests and private hosted
application smoke. Any proposal to expose an endpoint, enable a synthetic root
in Google Cloud or run an interactive hosted session is a scope change and must
wait for M3.4 approval.

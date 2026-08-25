# M1 threat model: common interaction and runtime spine

Status: Working baseline — founder review required for M1 acceptance

Version: 0.1.0

Last reviewed: 25 August 2026

## Scope

This threat model covers the M1 common contracts, schema catalogue, Rust
reference runtime, local append journal, safe health view and container binding.
It covers the local assurance profile only. It does not qualify an external
API, event broker, cryptographic identity, synthetic sign-in, multi-tenant
service or production deployment.

The model is a current baseline. New evidence may change likelihood, controls
or design; revisions must identify the resulting compatibility and security
effect.

## Protected assets

- correctness of command acceptance, refusal and duplicate handling;
- principal, purpose, environment, target and authority attribution;
- contract/schema identity and compatibility evidence;
- integrity and recoverability of delivery and audit state;
- confidentiality of credentials, restricted security state and source
  payloads;
- availability of safe refusal, readiness and diagnostic behaviour; and
- the distinction between executable evidence and broader maturity claims.

## Entry points and assumptions

The M1 executable entry point is a local command-line adapter reading one JSON
document from a named file. The invoking operator and local filesystem boundary
are prerequisites; the envelope is still treated as untrusted input. The
runtime emits a `C-003` outcome to standard output and appends redacted delivery
evidence under an explicitly configured state directory.

M1 does not claim that `C-002.security.authenticationContextRef` authenticates
the named principal. It tests the contract and decision path in an assurance
profile. Exposing the adapter through a network, browser or broker before M2
workload identity and an approved transport binding would violate this model.

## Threat register

| ID        | Threat                                                                     | Current M1 controls                                                                                                                          | Residual risk or next action                                                                                                     |
| --------- | -------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| `T-M1-01` | A caller forges actor, workload or authority metadata                      | Local operator boundary; strict schema; environment, target, principal-type, purpose and authority checks; no external listener              | Metadata is not authenticated. M2 must bind workload and actor context cryptographically before external exposure                |
| `T-M1-02` | Duplicate, delayed or retried delivery applies work twice                  | Required idempotency key; semantic fingerprint; exclusive state lock; durable append before successful outcome; restart/concurrency tests    | Single state directory is not a distributed consensus mechanism                                                                  |
| `T-M1-03` | One idempotency key is reused for different content                        | Fingerprint comparison; fail-closed `idempotency_conflict` outcome; no second operation                                                      | Cross-language canonical fingerprinting is not yet a public contract                                                             |
| `T-M1-04` | Contract downgrade or incompatible schema bypasses validation              | Exact contract/version checks; JSON Schema 2020-12 validation; compatibility descriptors; unsupported version refusal                        | Negotiation and rolling multi-version deployment are unqualified                                                                 |
| `T-M1-05` | Credential or secret enters a payload, fixture, journal or log             | Restricted field-name scan; schemas exclude credential fields; journal stores no envelope or payload; negative fixtures and disclosure tests | Content-based secret detection is incomplete; callers and future gateways remain responsible for classification and DLP controls |
| `T-M1-06` | Journal truncation or tampering hides or changes accepted work             | Parse every complete record; corruption makes readiness false; append and sync; no silent repair                                             | Journal is not tamper-evident and assumes control of the local filesystem; M6 must select qualified durable and evidence storage |
| `T-M1-07` | Concurrent writers race around idempotency                                 | Standard-library exclusive file lock around read/decide/append; macOS concurrency test                                                       | Windows, network filesystems and multiple replicas are not qualified                                                             |
| `T-M1-08` | Clock manipulation changes expiry decisions                                | RFC 3339 parsing; explicit injected clock in tests; future and expired requests refuse with bounded tolerance                                | Secure time and hosted clock monitoring belong to platform qualification                                                         |
| `T-M1-09` | Diagnostic output discloses payload, identity or internal state            | Safe codes, counts, digests and references only; no raw input in errors or journal                                                           | Process-level crash and third-party library diagnostics require continuing review                                                |
| `T-M1-10` | Malformed or oversized JSON consumes resources or causes ambiguity         | File-size limit, typed deserialisation, bounded schema fields and fail-closed errors                                                         | Streaming and hostile-network rate limiting are deferred                                                                         |
| `T-M1-11` | Compromised host or operator edits state or invokes the adapter            | Explicit local trust boundary, restricted state path and container hardening                                                                 | M1 cannot defend against an authorised host administrator; operator separation and tamper evidence remain later work             |
| `T-M1-12` | Dependency or build compromise changes validation/runtime behaviour        | Locked Rust and pnpm dependencies, CI checks, non-root minimal runtime image                                                                 | Signed provenance, SBOM policy and vulnerability response evidence remain M6 work                                                |
| `T-M1-13` | M1 evidence is presented as authenticated, hosted or production capability | Maturity labels, assurance-profile refusal boundary and explicit documentation                                                               | Founder/reviewer discipline remains necessary in public communication                                                            |

## Abuse and failure cases

The conformance suite must include:

- a structurally valid assurance command accepted once;
- the same delivery repeated before and after runtime restart;
- concurrent duplicate delivery;
- the same idempotency key with changed content;
- missing authority, purpose, classification or idempotency data;
- wrong environment, target, audience, principal type or purpose;
- expired, premature and unsupported-version commands;
- a payload containing a credential-like field;
- an incomplete or corrupted journal; and
- a read-only or unavailable state path.

An error before a durable decision must not report acceptance. A corrupted
journal makes interaction readiness false and requires operator action; the
runtime does not discard or rewrite history automatically.

## Security decisions deferred to later milestones

- authenticated workload and external-human bindings;
- synthetic root, certificate, signer, grant and session mechanisms;
- network API, event broker and browser delivery transports;
- distributed idempotency and ordering;
- encrypted, tamper-evident, replicated and backed-up evidence stores;
- hosted secrets, key custody and workload policy;
- ingress, rate limiting, denial-of-service protection and multi-tenancy; and
- production build signing, SBOM and release assurance.

Each later binding extends this threat model before it is accepted. It cannot
inherit an M1 assurance result merely because it uses the same JSON shape.

## Review gate

The threat model is ready for founder review when its controls have executable
positive and negative evidence on the supported native development profile and
inside the reference container. Founder review may accept it as the M1 baseline
or require a versioned revision. It is reviewed again before any external
transport or authenticated identity binding is enabled.

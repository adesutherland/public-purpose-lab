# M3.4 identity and resilience threat extension

Status: Implemented and evidenced development baseline; M3.4 closed

Version: 0.1.0

Last reviewed: 28 August 2026

Extends:
[M3 Scenario Director threat model](m3-threat-model.md),
[M3.2 presentation threat extension](m3-2-presentation-threat-extension.md) and
[M3.3 runtime-binding threat extension](m3-3-runtime-binding-threat-extension.md)

Related accepted decisions: ADR-0014 to ADR-0016 and ADR-0021 to ADR-0023

## Scope and claim boundary

This extension covers the M3.4 external-human adapter, component-owned browser
sessions, backend-only synthetic sign-in, target-owned replay state, managed
Cloud KMS signing adapter, three-workload event composition and restart-safe
local path.

The implementation remains synthetic-only and in development. Local tests did
not qualify Google, GKE, Cloud KMS or ingress; the bounded
[managed-hosted evidence](../evidence/m3-4-managed-hosted-identity-and-resilience.md)
later exercised those bindings. Neither result qualifies production recovery,
availability, legal, regulatory or non-synthetic-data controls. A future
managed-hosted activation remains unavailable unless its protected bindings
are present and admitted again.

## Refined trust boundaries

| Boundary                                 | Trust position                                                            | Required property                                                                                                       |
| ---------------------------------------- | ------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| Browser to application backend           | Untrusted input carried by an ordinary external-human application session | Opaque HttpOnly session, separate CSRF value, exact origin, current role-map version, expiry and revocation             |
| Google OIDC to `I-001` adapter           | Authentication evidence only                                              | Authorization Code, PKCE, state, nonce, issuer/audience/signature/time validation and exact issuer/subject role mapping |
| Director to Identity Broker              | Workload request, not grant authority                                     | Dedicated NATS identity, exact subject/action and configured actor/application/purpose checks                           |
| Identity Broker to Cloud KMS             | Narrow signing operation after policy approval                            | Pinned project/key version, canonical grant bytes, integrity checks, non-exportable key and exact KSA authority         |
| Identity Broker to application backend   | Protected signed grant delivery                                           | No browser route; application/audience/surface/scenario/environment/epoch/time binding                                  |
| Application backend to synthetic context | Receiving-component authority                                             | Independent signature validation, durable at-most-once establishment and live ordinary surface claim                    |
| Retained trust to activation state       | Managed trust survives ordinary `off`; runtime state does not             | Protected recovery evidence, no local-root promotion and explicit final trust-domain destruction                        |

External presenter, external surface operator, workload, synthetic actor and
business decision authority remain separate even when one person or one image
participates in several roles.

## Threat register extension

| ID        | Threat                                                                                   | Required M3.4 control                                                                                                                                              | Evidence or residual risk                                                                                              |
| --------- | ---------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------- |
| `T-M3-61` | OIDC response is replayed, redirected or substituted                                     | Exact HTTPS callback, state/nonce, PKCE, single-use durable flow, issuer/audience/signature/time validation and no redirect following during discovery/token calls | Live Google login passed; state/replay negatives remain in focused conformance tests                                   |
| `T-M3-62` | Authentication is mistaken for Lab authority                                             | Exact protected issuer/subject mapping, required role and audience, mapping version rechecked on each protected use                                                | Allow-list administration remains deliberately small                                                                   |
| `T-M3-63` | Session fixation or stolen browser state creates authority                               | New random session after login, hashes only at rest, HttpOnly/Secure/SameSite cookie, CSRF, exact origin, expiry, logout and role-version revocation               | Multi-node session storage is not selected                                                                             |
| `T-M3-64` | Provider token, code, cookie or grant reaches frontend, event evidence or logs           | Provider material ends at backend; browser receives only privacy-minimised context; disclosure scan covers bundles, events and output                              | Hosted disclosure scan passed; support access to host memory remains administrative capability                         |
| `T-M3-65` | Synthetic grant is issued for an unapproved actor, role, application or purpose          | Actor registry, relationship/consent assertions, `AZ-001`, mandatory obligations and canonical plan persisted before signing                                       | External policy product remains replaceable and not selected here                                                      |
| `T-M3-66` | KMS becomes a general signing oracle or wrong key/project is used                        | No signing endpoint, canonical prepared-plan API, exact project-scoped key version, response name/protection/checksum verification and pinned public key           | Exact workload/key and Data Access audit passed; a compromised broker still requires ordinary containment              |
| `T-M3-67` | Grant is replayed or established in another application, surface, session or environment | Signed bindings plus independent target replay store, live surface claim and conflict-safe establishment operation                                                 | Store loss fails availability and does not recreate authority                                                          |
| `T-M3-68` | Restart or browser refresh silently recreates expired authority                          | Durable component-owned state, operational-time validation, no URL/cursor authority and current mapping check                                                      | First binding is single-instance, not HA                                                                               |
| `T-M3-69` | Stop, reset or supersession leaves a synthetic actor active                              | `I-005` termination event and component-local clearing for every binding in the Demonstration Session                                                              | Broker loss can delay termination, so affected interaction must stop safely                                            |
| `T-M3-70` | One synthetic actor is implicitly shared across different scenario surfaces              | Per-surface ordinary sessions and signed actor/surface bindings; assurance fixture uses distinct audience and reviewer actors                                      | The first runtime shares one Presentation Gateway backend; independent application gateways remain a later composition |
| `T-M3-71` | Local test identity or local signer is relabelled as managed                             | Closed runtime profile, hosted refusal, managed trust/custody checks and no local private-key mount in managed overlay                                             | Configuration admission still requires hosted negative testing                                                         |
| `T-M3-72` | Single broker loss is mistaken for resilient delivery                                    | Explicit one-server profile, file store, durable consumers, bounded redelivery and component idempotency; readiness exposes the limitation                         | No availability SLO or transparent broker-volume recovery is claimed                                                   |

## Required evidence

Local and container evidence must cover authorised presenter and operator
sessions, wrong origin and CSRF, expiry, mapping change, durable restart,
multi-surface actors, duplicate/conflicting/cross-environment grants, stop/reset
termination, broker restart and disclosure scanning. The managed binding must
add live Google OIDC, exact GKE workload identity, Cloud KMS signature and wrong
key refusal, HTTPS/SSE behaviour, immutable image, automatic expiry, cost and
conclusive teardown evidence.

## Residual risk and review point

The required local and managed-hosted evidence passed at development maturity.
Automatic-expiry callback execution remains an M3.5 case; M3.4 proved arming
before creation and reconciliation by normal off.

The current design is intentionally one replica per workload and one
file-backed broker. It has no general identity administration, refresh tokens,
high availability, authoritative audit store, non-synthetic-data authority or
production recovery qualification. Review this threat model before another
identity provider, application gateway, replica, policy product, real data or
unsupervised hosted operation is introduced.

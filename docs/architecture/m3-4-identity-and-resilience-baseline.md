# M3.4 identity and resilience baseline

Status: Accepted for implementation
Date: 2026-08-28

## Purpose and claim boundary

M3.4 turns the M3.3 walking skeleton into a restartable, authenticated
synthetic demonstration path. It proves that an authorised external presenter
can direct a scenario, a separately authorised surface operator can connect an
application, and a named synthetic actor can be signed into that application
through a protected backend-only event path.

The result remains an in-development, synthetic-only demonstrator. It is not a
production identity system, compliance mechanism, legal authority, clinical or
professional decision system, high-availability service or permission to use
real information.

## Logical runtime

| Responsibility                          | M3.4 binding                                                                                                                 | Fail-closed dependency                                                                                     |
| --------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| External presenter and surface operator | `I-001` adapter; Google OIDC in managed profiles, explicit test identity locally                                             | Verified issuer/subject, audience, environment role map and current mapping version                        |
| Browser application session             | Component-owned durable opaque session, secure cookie in hosted use, exact origin and CSRF                                   | Writable consistent store, unexpired session, current role mapping                                         |
| Workload identity                       | Per-workload NATS identity locally; dedicated Kubernetes service account and GKE Workload Identity Federation in managed use | Exact workload, audience and action scope                                                                  |
| Synthetic trust issuer                  | Local environment Ed25519 signer or retained managed Cloud KMS issuer                                                        | Compatible trust record, current epoch, protected signer and policy/configuration                          |
| Synthetic application sign-in           | `I-004` protected event to target backend; target owns `I-005` establishment and application binding                         | Signature, environment, application, surface, scenario session, expiry and durable replay state            |
| Presentation delivery                   | Same-origin SSE plus CSRF-protected outcome POST                                                                             | Ordinary application session, active synthetic binding, registration generation/lease and current scenario |
| Component events                        | Single-server file-backed NATS JetStream with separate credentials and durable consumers                                     | Broker/stream availability and component idempotency                                                       |
| Control state                           | Existing component-owned SQLite stores                                                                                       | Consistent store and expected revision/state                                                               |

## Required invariants

1. External human, workload, synthetic actor and accountable business authority
   are distinct identities.
2. Authentication never creates a Lab role or business authority by itself.
3. A synthetic grant is environment-, epoch-, application-, audience-,
   surface- and Demonstration-Session-bound, short-lived and established at
   most once.
4. Different applications in one scenario may use different synthetic actors;
   one binding cannot authorise another application.
5. No browser receives infrastructure credentials, provider tokens, a signed
   grant or a usable synthetic session reference.
6. Restart, refresh and reconnect preserve only durable, still-valid authority.
   Memory, URL state and browser-supplied cursors cannot recreate authority.
7. Stop, reset, supersession, expiry, logout, role removal and trust revocation
   terminate the affected binding. A successor session requires fresh grants.
8. Duplicate or delayed events reconcile to a prior safe result; conflicting
   reuse, unknown versions and incomplete protected state are refused.
9. Presentation progress remains distinct from software health, human
   attention, business completion and compliance evidence.

## Profiles

| Profile                               | External identity                                                | Synthetic signer                                    | Persistence                                                              | Claim                                               |
| ------------------------------------- | ---------------------------------------------------------------- | --------------------------------------------------- | ------------------------------------------------------------------------ | --------------------------------------------------- |
| Native / Compose / Minikube assurance | Explicit local test identity                                     | Environment-generated local Ed25519 issuer          | Component-owned local volume                                             | Deterministic development assurance only            |
| Private hosted validation             | Google OIDC configuration self-test; no public endpoint required | Retained Cloud KMS issuer through workload identity | Activation-scoped component state                                        | Private managed-binding validation                  |
| Shared hosted demonstration           | Google OIDC for named authorised users                           | Same managed issuer; no local private root          | Activation-scoped component state plus protected retained trust/evidence | Short-lived supervised synthetic demonstration only |

No profile is promoted by relabelling. A managed profile rejects local keys and
the local identity adapter even if all other tests pass.

## Acceptance cases

M3.4 implementation evidence must include:

- authorised presenter and separately authorised surface operator;
- authenticated but unmapped external identity and changed/removed mapping;
- session fixation, CSRF, wrong origin, expiry, logout, restart and refresh;
- successful backend-only synthetic sign-in and two distinct synthetic
  actor/surface bindings in one scenario fixture, alongside the M2
  cross-application conformance case;
- duplicate, conflicting, expired, premature, wrong-audience,
  wrong-application, wrong-surface and cross-environment grants;
- Director, identity broker, Presentation Gateway and broker restart;
- SSE reconnect with at most the latest still-valid cue for the current
  registration generation and no generic browser-cursor replay;
- stop, reset and successor-session revocation;
- unsupported/expired/delayed/duplicate cues and safe checkpoint results;
- local-container and Minikube parity, immutable hosted image, managed signer
  and workload-identity evidence, authorised activation, automatic expiry and
  conclusive teardown; and
- disclosure scanning plus explicit single-server and synthetic-only limits.

M3.4 closes only when code, tests, operational evidence and limitations agree.
Bindings and numerical limits are a current baseline and may be revised by a
later recorded decision as the framework develops.

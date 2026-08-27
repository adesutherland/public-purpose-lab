# M3.2 presentation and hosted-binding threat extension

Status: Accepted M3.2 binding threat baseline; implementation evidence pending

Version: 0.1.0

Last reviewed: 27 August 2026

Extends: [M3 Scenario Director threat model](m3-threat-model.md)

Related accepted logical specifications:
[`CTL-02`](../logical/components/ctl-02-presentation-gateway-and-screen-registry.md),
[`P-001`](../logical/contracts/presentation/p-001-presentation-capability-manifest.md)
to
[`P-004`](../logical/contracts/presentation/p-004-presentation-cue-outcome.md)
and ADR-0013 to ADR-0016

## Scope and assurance claim

This extension covers surface capability admission, authenticated registration,
semantic cue delivery, presenter sign-in, the backend/browser boundary, first
component-event transport and the cost-controlled Google Cloud preview binding.

It does not qualify a shared hosted demonstration, managed identity service,
production environment, real-data path or compliance control. Acceptance
creates no Google Cloud resource. The selected service choices remain subject
to implementation and conformance evidence.

The accepted M3.1 invariants continue unchanged: a cue has presentation
authority only, browser routes are not contract meaning, business actions are
separately authorised, reset creates a successor Demonstration Session and
scenario logical time cannot affect protected security time.

## New trust boundaries

| Boundary                                        | Trust position                                                              | Required control                                                                                                                                        |
| ----------------------------------------------- | --------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| External presenter to Google OIDC               | External identity proof, not Lab authorisation                              | Server flow, state/nonce, exact redirect and token validation; stable issuer/subject mapping; minimal claims.                                           |
| Presenter browser to Director backend           | Untrusted client over an ordinary application session                       | Secure backend session, CSRF/origin controls, current authorisation for every protected action, no provider token as application authority.             |
| Presentation browser to application backend     | Untrusted rendering client with no broker, signer or private-store access   | Same-origin protected session, bounded SSE delivery and authenticated/CSRF-protected outcome POST; safe reconnect.                                      |
| Application backend to `CTL-02`                 | Authenticated workload and current application-session context              | `I-002`, current policy, exact environment/session/slot binding, least-privilege event subjects and no reusable browser credential in messages.         |
| `CTL-01`/`CTL-02` to NATS JetStream             | Replaceable at-least-once component transport                               | TLS/authentication, subject-level authorisation, bounded streams/consumers, expiry, idempotent receivers and no claim of global exactly-once semantics. |
| GKE workload to Google Cloud APIs               | Federated workload identity, distinct from node or operator identity        | Dedicated Kubernetes service accounts, Workload Identity Federation, resource-scoped IAM and no exported service-account key.                           |
| Hosted identity signer to Cloud KMS             | Managed signing operation with non-exportable private key material          | Dedicated key/purpose, environment-scoped signing workload, minimal `useToSign`, public-key/version evidence, rotation/revocation and audit.            |
| Operator/pipeline to Google Cloud control plane | Elevated lifecycle authority, not presenter/business authority              | Named operator approval, short-lived deployment federation, bounded role, immutable input, expiry, conclusive teardown and no stored personal key.      |
| Retained `off` resources                        | Residual cost, endpoint, trust and evidence surface after workload teardown | Explicit inventory, allow-list, endpoint absence checks, cost reconciliation, custody/retention and visible cleanup exceptions.                         |

Co-deploying `CTL-01` and `CTL-02` does not combine their authority. Sharing a
Google Cloud project or Kubernetes namespace does not by itself prove workload
identity; project-level workload-identity sameness and broad IAM grants must be
avoided or constrained.

## Security objectives added by M3.2

M3.2 must:

1. bind each active surface generation to one environment, Demonstration
   Session, application and package-declared surface slot;
2. authenticate a surface through its application backend without exposing
   identity grants, application credentials or broker access to the browser;
3. keep semantic presentation control separate from browser routing and
   business authority;
4. expire and supersede registrations and cues under protected operational
   time, including across reconnect and restart;
5. prevent stale surfaces, delayed events and prior-session state from
   controlling a successor Demonstration Session;
6. authenticate presenters externally while authorising their Lab role and
   actions locally and explicitly;
7. make component transport at-least-once behaviour visible and require
   receiver idempotency rather than claiming end-to-end exactly once;
8. use non-exported managed signing material and federated workload identities
   for any shared hosted profile;
9. make hosted activation named, bounded, automatically expiring and
   conclusively reconcilable; and
10. expose trust profile, residual resources and gross/credit/net cost without
    leaking protected configuration.

## Threat register extension

| ID        | Threat                                                                                           | Required M3.2 control                                                                                                                                                   | Evidence gate                                                                       |
| --------- | ------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| `T-M3-26` | A malicious or copied frontend registers as an approved surface                                  | Backend workload authentication, admitted application release/manifest, current application session where required, policy decision and exact slot binding              | Untrusted workload, release and manifest negative tests                             |
| `T-M3-27` | One surface occupies or impersonates another scenario slot                                       | Package-declared slot, one current authoritative binding per slot, expected revision, atomic supersession and safe conflict response                                    | Concurrent-registration and cross-slot tests                                        |
| `T-M3-28` | A stale browser reconnects and receives current or successor-session cues                        | Current backend session proof, new connection generation, protected lease, successor-session isolation and no local-storage restoration                                 | Disconnect, reconnect, reset and delayed-connection tests                           |
| `T-M3-29` | A delayed cue applies after expiry, stop, reset or registration supersession                     | Expiry at gateway and surface, current session/revision/generation validation and latest-valid-cue-only reconciliation                                                  | Queue-delay, stopped-session, reset and superseded-generation tests                 |
| `T-M3-30` | Duplicate delivery applies presentation state repeatedly or with changed content                 | Immutable cue digest, scoped idempotency, surface commit-once result and conflict refusal                                                                               | Identical and conflicting duplicate tests                                           |
| `T-M3-31` | A forged or stale outcome falsely concludes a cue                                                | Authenticated backend source, cue/digest/session/slot/revision/generation match and terminal transition validation                                                      | Wrong-source, late, conflicting and restart reconciliation tests                    |
| `T-M3-32` | Cue context is interpreted as a URL, script or hidden business command                           | Closed semantic view/context schemas, prohibited-content validation, frontend resolver allow-list and no business-command capability at the surface channel             | URL, DOM, script, command and unknown-field negative fixtures                       |
| `T-M3-33` | Browser access to the broker bypasses gateway policy or exposes component events                 | Backend-mediated browser channel only; broker network/credentials unavailable to frontend; origin/session/CSRF controls                                                 | Browser bundle, network policy, credential and direct-connection tests              |
| `T-M3-34` | Reconnect identifiers or event-stream cursors become reusable cross-session credentials          | Opaque non-authority identifiers, backend session revalidation, strict generation/session binding and no generic broker sequence or bearer token in the browser         | Copied cursor/registration and wrong-session reconnect tests                        |
| `T-M3-35` | Presenter Google identity is treated as automatic Lab or business authorisation                  | Exact issuer/subject authentication followed by environment role mapping and `AUT-01`; receiving-component enforcement for every protected action                       | Known-but-unauthorised, removed-role and wrong-environment tests                    |
| `T-M3-36` | OIDC login is replayed, redirected or fixed to an attacker-controlled session                    | Server flow with exact redirect URI, state, nonce, PKCE where supported, ID-token validation, session rotation and secure cookie/CSRF controls                          | State/nonce/redirect/session-fixation negative tests                                |
| `T-M3-37` | A compromised component publishes or consumes broader event subjects                             | Per-workload NATS identity, allow-listed publish/subscribe subjects, TLS, bounded payloads, receiver authority checks and audit                                         | Subject-permission and receiving-component refusal tests                            |
| `T-M3-38` | At-least-once broker behaviour is mistaken for end-to-end exactly-once processing                | Explicit duplicate semantics, durable operation identity, receiver idempotency, uncertain reconciliation and no success inferred from broker acknowledgement            | Lost-acknowledgement, redelivery, restart and conflicting-content tests             |
| `T-M3-39` | GKE workloads inherit broad node/project credentials or collide across clusters                  | Workload Identity Federation, dedicated namespace/service accounts, resource-level IAM, cluster/project scoping and no host-network exception for application workloads | Effective-IAM, metadata access and cross-cluster identity tests                     |
| `T-M3-40` | Hosted signer becomes a general signing oracle or its private key is exported                    | Cloud KMS asymmetric key, non-exportable private material, dedicated signing workload, exact purpose/audience policy, minimal signing IAM, rotation and audit           | Wrong-workload/purpose/input refusal, public-key verification and rotation tests    |
| `T-M3-41` | A local-synthetic root is copied or relabelled into a hosted/shared environment                  | Hosted profile requires `managed`; root/profile readiness visibly fails closed; bootstrap accepts only the declared KMS-backed environment issuer                       | Local-key injection, wrong-project/key/version and cross-environment grant tests    |
| `T-M3-42` | A deployment pipeline or operator credential is stolen or used as presenter/business authority   | Short-lived workload federation, named approval, environment-scoped deploy role, protected branch/artifact digest and principal-type separation                         | Token lifetime/audience, least-privilege and denied presenter/business action tests |
| `T-M3-43` | Automatic expiry fails and leaves a reachable or materially charging environment                 | Independent expiry controller/workflow, idempotent off operation, endpoint/workload/resource inventory, alert and conclusive teardown reconciliation                    | Interrupted expiry, repeat-off, residual-endpoint and cost inventory tests          |
| `T-M3-44` | Budgets or credits are mistaken for a hard stop or zero-cost proof                               | Off lifecycle is primary control; gross usage, credits and net cost separated; alerts/caps defence in depth; persistent resources explicitly costed                     | Activation cost report and over-threshold exercise without reliance on credit       |
| `T-M3-45` | Logs, event streams, frontend diagnostics or cloud audit exports leak security or route material | Contract allow-lists, structured redaction, restricted support access, bounded retention and automated scans across browser, broker, application and cloud evidence     | End-to-end disclosure scan and restricted-diagnostic access test                    |

## Required misuse and recovery cases

The M3.2 evidence plan adds:

- two surface slots active simultaneously with different synthetic actors and
  application sessions;
- a copied surface identifier, wrong application backend and wrong
  Demonstration Session registration;
- two registrations racing for one slot, followed by deterministic
  supersession or refusal;
- disconnect before delivery, reconnect with a new generation and a late old-
  generation outcome;
- cue redelivery before and after acknowledgement, changed content under the
  same identity and gateway restart during uncertainty;
- a cue containing a route, URL, credential-like value, executable content and
  a business-action-shaped field;
- an `applied` presentation result alongside an unsatisfied business
  checkpoint;
- an authenticated but unauthorised Google user, OIDC state/nonce failures and
  revoked local presenter role;
- direct browser attempts to reach NATS and attempts by one component to use
  another component's subjects;
- hosted workload access with the correct and incorrect Kubernetes service
  account, namespace, cluster and IAM resource;
- attempts to use the managed signer for the wrong purpose, environment,
  audience and payload;
- attempted hosted bootstrap with local-synthetic or another environment's
  trust material; and
- interrupted activation/expiry/deactivation followed by endpoint, retained-
  resource and gross/credit/net cost reconciliation.

## Binding acceptance gates

The founders accepted these M3.2 design gates on 27 August 2026:

1. `CTL-02` and `P-001` to `P-004` semantics;
2. the event transport and its subject, retention, expiry, retry and dead-
   letter profile;
3. presenter authentication and local authorisation mapping;
4. backend/browser surface channel and reconnect profile; and
5. the first Google Cloud services, operator federation, managed signer,
   retained `off` inventory and infrastructure-as-code ownership.

Implementation must now supply the positive, negative, restart, disclosure and
profile evidence specified in this extension. Design acceptance is not an
executable-control claim.

Before any M3.4 shared hosted demonstration, implementation evidence must show
the managed trust, presenter/workload identity, protected state and automatic
expiry paths end to end. A disposable infrastructure spike supplies lifecycle
evidence only and cannot satisfy that gate.

## Residual risk and review outcome

The main residual risks are still physical: frontend session implementation,
OIDC client registration, broker configuration, state atomicity, exact resource
limits, Cloud KMS certificate/issuer construction, pipeline IAM and teardown
coverage. Accepted ADRs narrow those choices but do not close the risks without
tests and recovery evidence.

The founders accepted this document as the M3.2 binding threat baseline on 27
August 2026. Acceptance applies only to the design maturity stated in the
associated ADRs; it does not claim implemented controls or a production
security qualification.

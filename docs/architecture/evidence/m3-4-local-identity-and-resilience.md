# M3.4 local identity and resilience implementation evidence

Status: Local implementation evidence complete; managed-hosted supplement closes M3.4

Evidence date: 28 August 2026

## Claim and boundary

This record demonstrates the M3.4 development baseline natively and in the
local Minikube profile. It covers durable external application sessions, a
separate Identity Broker, backend-only synthetic grant delivery, two distinct
actor/surface bindings, restart/reconnect and safe stop/reset behaviour using
synthetic information.

This local record does not itself demonstrate Google authentication, a live
Cloud KMS signature, shared HTTPS ingress, immutable hosted-image execution,
cloud expiry or conclusive cloud teardown. Those bounded results are recorded
in the later
[managed-hosted supplement](m3-4-managed-hosted-identity-and-resilience.md).
Neither record is production, privacy, legal, regulatory, availability or
non-synthetic-data assurance.

## Implemented path

- Director, Identity Broker and Presentation Gateway are separate runtime
  workloads with separate NATS identities and durable component state.
- External-human and synthetic-human authority remain separate. Local profiles
  expose a labelled test identity; managed profiles use Google OIDC only.
- Browser sessions retain hashes of opaque session and CSRF credentials, role
  mapping version, audience and expiry. They do not retain raw credentials,
  provider tokens or signed grants.
- The Director publishes a surface-bound identity request. The Identity Broker
  issues a short-lived signed grant and delivers it only to the target backend.
  The browser never receives it.
- The target validates environment, epoch, application, audience, surface,
  Demonstration Session, signature, time and durable replay state before
  establishing the binding at most once.
- `synthetic-audience-user` is bound to `audience-display` and
  `synthetic-reviewer` to `reviewer-workbench` in independent browser sessions.
- Stop, reset and successor creation remove prior synthetic authority; a
  successor begins without a checkpoint or logical-time state.

## Source checks

The complete repository check passed after implementation:

- 21 component and 40 contract-family catalogue entries;
- 20 schemas, 40 fixtures and 20 compatibility descriptors;
- 96 local Markdown files checked for links;
- Prettier, TypeScript typechecks and 10 frontend package tests;
- Clippy with warnings denied;
- 57 Rust unit tests, including 19 IAM tests and four M3 runtime tests;
- existing M1 and M2 end-to-end regression checkers; and
- complete Rust and frontend production builds.

The scenario package is `1.1.0`, with package digest
`566ab18f473016e8415a1664ed2c7e882508f2683429a685fd9afea190307289`
and scenario digest
`88008cb19a26292da8439fde6425f0ff17dee5e24e97d740db745632cd704caf`.

Focused negative and recovery cases cover raw-credential non-persistence,
restart, role mapping, CSRF, expiry, grant conflict, cross-environment,
wrong-surface/session/application binding, managed signing idempotency, KMS
metadata/public-key disagreement, OIDC state reuse and exact HTTPS callback
binding.

## Runtime evidence

The native path completed the full secure smoke. The first Minikube activation
built image identity
`sha256:7f033f8d047288c369d2b1a9c384a55f41fe3f538264bc357916723ee9208114`,
rolled out NATS and all three workloads, and completed the same smoke. NATS and
all three workloads were then deliberately restarted and the smoke completed a
second time.

Both Minikube runs observed:

- wrong CSRF refused with `401`;
- audience actor `synthetic-audience-user`;
- workbench actor `synthetic-reviewer`;
- bounded `presentation-cue-delay-armed` fault;
- semantic SSE view `assurance-welcome`;
- satisfied presentation checkpoint;
- prior session `superseded`; and
- a successor with no checkpoint and logical time not initialised.

The Kubernetes profile used one file-backed JetStream server, three component
workload credentials, TLS, four component/PVC state boundaries and no Ingress
or LoadBalancer. The VM was stopped after testing and its local assurance state
was retained. A pre-M3.4 trust directory was explicitly refused rather than
silently gaining a new Identity Broker credential.

## Managed-hosted implementation boundary

The managed adapter uses Google discovery and Authorization Code with PKCE,
state and nonce. It maps exact issuer/subject values through a protected,
versioned audience/role map and creates an application-owned session. The
managed Identity Broker calls the pinned Cloud KMS Ed25519 key through
application-default/workload identity, checks enabled state, algorithm,
software protection, public-key checksum and exact public trust fingerprint at
startup, and validates response integrity for every signature.

The public managed-hosted Kustomize overlay renders three dedicated Kubernetes
service accounts. Only `m3-identity-broker` has projected Kubernetes identity;
Director and Presentation automount no token. The protected OpenTofu definition
validates an issuer-key-only `roles/cloudkms.signerVerifier` grant for that
exact namespace/service-account principal.

No key, OIDC client, endpoint or cloud workload was created by this
implementation run. The retained root/issuer certificate bootstrap, protected
configuration, live Google/KMS cases, immutable image, shared HTTPS activation,
automatic expiry, cost and teardown evidence remain founder-reviewed gates.
The OCI Compose restart test is defined in CI but awaits the published branch's
hosted run because no local Docker-compatible runtime was installed.

## Result

This record completed M3.4's local implementation evidence. The separately
approved and published managed-hosted evidence run later satisfied the bounded
Google, KMS, HTTPS, workload, restart and teardown gates. The two records close
M3.4 only at in-development, synthetic-only maturity.

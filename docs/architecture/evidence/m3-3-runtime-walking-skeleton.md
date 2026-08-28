# M3.3 runtime walking-skeleton evidence

Status: M3.3 implementation evidence completed; synthetic-only and in development

Evidence date: 28 August 2026

Decision authority: Public Purpose Lab founders

## Claim boundary

This record covers an in-development, synthetic-only walking skeleton. It does
not qualify a shared service, managed identity, real information, production
operation, legal or compliance authority, high availability, backup/restore or
business completion.

The accepted baseline and closure criteria remain in the
[M3.3 runtime walking-skeleton baseline](../m3-3-runtime-walking-skeleton-baseline.md).

## Implemented scope

The repository now contains:

- closed D-001 to D-004 and P-001 to P-004 JSON schemas, examples, negative
  fixtures, descriptors and compatibility manifests;
- RFC 8785 JSON canonicalisation and SHA-256 package admission;
- Rust and TypeScript contract types;
- separate `CTL-01` and `CTL-02` component-owned SQLite stores with durable
  inbox/state/outbox decisions;
- lifecycle, presentation checkpoint, manual-step logical time, semantic reset
  and a one-shot bounded cue-delay fault;
- a TLS/NKey NATS JetStream `INT-01` adapter with separate workload
  permissions, durable consumers and explicit acknowledgement;
- one image with separate Scenario Director and Presentation Gateway modes;
- local development-assurance session, Director, Presentation and Workbench
  surfaces, with no browser access to the broker or databases; and
- native, OCI Compose, Minikube and ingress-free private-hosted manifests or
  adapters.

The canonical package digest observed in repository checks and runtime
admission is:

```text
b956e12c341283d683269c3dcabfa99f7297196814132d584bb673de4d8b9198
```

The canonical scenario content digest is:

```text
271b3dbafbcfa7e002ed8b7bde1ee8f437587baea4b8c60fafbc50ec049a9ed6
```

## Evidence observed

| Gate                                         | Result                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       | Boundary                                                                                                            |
| -------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| Repository schema and package checks         | Passed for 20 schemas, 40 fixtures and 20 descriptors; package and scenario digests reproduced during implementation. The exact closure source `a3dbb4eafdd80b1b57ea34d840e70d606a9a1e40` passed the published main-branch checks in [GitHub Actions run 33175352474](https://github.com/adesutherland/public-purpose-lab/actions/runs/33175352474).                                                                                                                                                                                                                                                                         | Current in-development baseline only.                                                                               |
| Rust component and integration tests         | Strict Clippy and the full workspace passed, including 4 `CTL-01`, 5 `CTL-02` and 12 `INT-01` tests for lifecycle, idempotency, reset, fault and broker rules.                                                                                                                                                                                                                                                                                                                                                                                                                                                               | Hosted and multi-replica operation remain outside this unit evidence.                                               |
| Frontend type, unit and release build checks | Passed.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      | Browser accessibility and disclosure review remains bounded to the current shell.                                   |
| Secure native path                           | Passed over TLS/NKey NATS: registration, logical time, prepare/start, one-shot fault, SSE cue, P-004, presentation checkpoint, stop and successor reset.                                                                                                                                                                                                                                                                                                                                                                                                                                                                     | Development-assurance session only.                                                                                 |
| Native restart                               | Passed: successor session, prior checkpoint history and empty pending outbox were recovered.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 | Single-instance SQLite only.                                                                                        |
| Minikube path                                | Passed from local image ID `sha256:109267d598de0015e088509041327c6ca6a6707bdea11a7e25bc5c5747ee6dbe`; both running-Pod image IDs were read back and the full path passed before and after broker and application restarts.                                                                                                                                                                                                                                                                                                                                                                                                   | This local image ID is not the future immutable Artifact Registry manifest digest.                                  |
| Kubernetes exposure and secret boundary      | Passed: ClusterIP only, no Ingress; the root private key was absent, and workloads received only their own seed plus the public root certificate.                                                                                                                                                                                                                                                                                                                                                                                                                                                                            | Local-synthetic Minikube environment only.                                                                          |
| Local OCI Compose                            | Passed in [GitHub Actions run 33099143491](https://github.com/adesutherland/public-purpose-lab/actions/runs/33099143491) at source revision `667ee8d3ec2ab2ab7334e19ed956ebf617873e65`: the secure TLS/NKey path passed before and after broker and application restarts, and disposable volumes were removed.                                                                                                                                                                                                                                                                                                               | Independent Linux Docker Compose evidence; Docker remains an available portable binding, not a mandated local host. |
| Private-hosted runtime profile               | Source `a3dbb4eafdd80b1b57ea34d840e70d606a9a1e40` was published as immutable image digest `sha256:73bf307891301f9a0ce175b89d40b9f1e74f5196164eca517e3684a42e213082`. In protected `on` run `33177789501`, both workloads passed liveness and contract/package self-tests with zero restarts; Pod image IDs matched the digest; the namespace contained zero Services, Ingresses and Secrets; both modes returned expected `503` readiness and `401` development-session refusal. Protected `off` run `33178599453` and independent inventory then proved no disposable activation resources or pending expiry task remained. | Ingress-free, expected-not-ready Google Cloud smoke only; no interactive, shared or managed-trust path was enabled. |

The positive native and Minikube path produced a `satisfied` presentation
checkpoint, retained the prior session as `superseded`, created a distinct
successor with no inherited checkpoint and left successor logical time
uninitialised. This is presentation-control evidence only.

## Defects found and corrected during evidence work

The evidence cycles found and corrected issues before closure:

- the Presentation consumer initially filtered cue subjects and could not
  receive D-003 control requests;
- the one-shot fault required an automatic expiry;
- repeated logical-time advances were initially bounded per call instead of
  cumulatively against the package declaration;
- duplicate cue delivery could have been rebroadcast to the browser;
- the Kubernetes NATS root-certificate key name did not match its mount; and
- human-readable NATS storage quantities rendered invalid server limits and
  were replaced by exact byte values;
- the application image build omitted documentation required by the contract
  checker; and
- Minikube could retain an old mutable-tag image after a failed or unchanged
  deployment, so startup now detects build errors, forces a rollout and checks
  each running Pod's image ID; and
- Docker Compose ignored requested ownership for bind-backed secrets, so the
  portable environment now combines an owner-only parent directory with
  container-readable mounted files while retaining the unmounted root private
  key at owner-only mode; and
- the native host layout's coexistence port was initially reused inside the
  portable container, so native now defaults to `4223` and portable to the
  standard container port `4222`; and
- the first protected Google Cloud activation created its private cluster but
  stopped before `kubectl apply` because the runner lacked the required GKE
  authentication plugin. Explicit `off` removed the activation completely;
  the official setup action was then pinned to install only that plugin before
  the successful fresh activation.

These corrections illustrate why the current mechanisms remain a revisable
baseline even when their accepted invariants are stable.

## Closure result

The accepted gates are conclusive for the bounded M3.3 claim:

1. repository, Rust, TypeScript, package and disclosure checks passed;
2. native, independent OCI Compose and Minikube paths passed, including
   restart/reconciliation and exact local image read-back;
3. the reviewed source was published as one immutable image and the private
   Google Cloud workloads read back that exact digest;
4. hosted liveness and contract/package self-tests passed with no endpoint,
   Service, Ingress or Kubernetes Secret;
5. hosted interactive readiness and the local development-session adapter
   failed closed with the expected `503` and `401` responses;
6. explicit `off` and independent inventory found no disposable activation
   resource or pending expiry task; and
7. the protected record captures activation duration and a conservative
   combined gross forecast ceiling of USD 0.50 before credits. Provider billing
   actuals, credits and net amount were still delayed and are recorded as
   pending, not zero, under the established billing-lag rule.

M3.3 is therefore closed as an in-development, synthetic-only walking-skeleton
baseline. The billing read-back remains an operational follow-up and cannot be
used to restate the forecast as actual cost. Managed presenter and workload
identity, synthetic application sign-in and the fuller security/resilience
suite remain M3.4 work.

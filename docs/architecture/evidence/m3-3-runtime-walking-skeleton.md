# M3.3 runtime walking-skeleton evidence

Status: Partial implementation evidence; M3.3 not closed

Evidence date: 27 August 2026

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

## Evidence observed so far

| Gate                                         | Result                                                                                                                                                         | Boundary                                                                                                               |
| -------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| Repository schema and package checks         | Passed for 20 schemas, 40 fixtures and 20 descriptors; package and scenario digests reproduced.                                                                | Exact final source revision still to be recorded.                                                                      |
| Rust component and integration tests         | Passed for the implemented `CTL-01`, `CTL-02` and `INT-01` cases, including lifecycle, idempotency, reset, fault and broker rules.                             | Final full-workspace rerun remains required after documentation completion.                                            |
| Frontend type, unit and release build checks | Passed.                                                                                                                                                        | Browser accessibility and disclosure review remains bounded to the current shell.                                      |
| Secure native path                           | Passed over TLS/NKey NATS: registration, logical time, prepare/start, one-shot fault, SSE cue, P-004, presentation checkpoint, stop and successor reset.       | Development-assurance session only.                                                                                    |
| Native restart                               | Passed: successor session, prior checkpoint history and empty pending outbox were recovered.                                                                   | Single-instance SQLite only.                                                                                           |
| Minikube path                                | Passed with separate workloads, identities and `ReadWriteOnce` claims; the full path passed before and after NATS and both application workload restarts.      | A final exact-image rerun remains required.                                                                            |
| Kubernetes exposure and secret boundary      | Passed: ClusterIP only, no Ingress; the root private key was absent, and workloads received only their own seed plus the public root certificate.              | Local-synthetic Minikube environment only.                                                                             |
| Local OCI Compose                            | Manifest and image build supplied; dynamic Compose run not yet evidenced because no independent compatible Compose runtime was installed on the evidence host. | Closure gate remains open.                                                                                             |
| Private Google Cloud                         | Ingress-free, fail-closed overlay and lifecycle integration drafted in the private hosting repository.                                                         | Must use the reviewed immutable image on the protected hosted path; execution, cost and teardown evidence remain open. |

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
  were replaced by exact byte values.

These corrections illustrate why the current mechanisms remain a revisable
baseline even when their accepted invariants are stable.

## Remaining closure gates

M3.3 remains open until one reviewed source revision and immutable image digest
have conclusive evidence for all of the following:

1. full repository, Rust, TypeScript and static disclosure checks;
2. native, independent local-container and Minikube parity;
3. application and broker restart/reconciliation from that image;
4. private Google Cloud liveness and contract/package self-test with no public
   endpoint;
5. expected `503` interactive readiness and `401` development-session refusal
   in the hosted profile;
6. explicit hosted teardown with no disposable activation resources; and
7. activation duration, gross usage, credits and net cost recorded privately
   without using credits to conceal gross cost.

Completion of those gates qualifies only M3.3. Managed presenter and workload
identity, synthetic application sign-in and the fuller security/resilience
suite remain M3.4 work.

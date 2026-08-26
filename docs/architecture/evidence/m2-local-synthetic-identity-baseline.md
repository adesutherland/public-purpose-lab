# M2 evidence: local-synthetic identity baseline

Status: Accepted M2 local-synthetic development-assurance baseline

Evidence date: 26 August 2026

Founder acceptance date: 26 August 2026

## Scope

This record covers the M2 isolated local-synthetic profile:

- `IAM-01` environment identity, workload context, demonstration grants and
  synthetic-session outcomes;
- the bounded in-process `AUT-01` policy adapter and `AZ-001` decisions;
- canonical `I-001` to `I-005` and `AZ-001` schemas and shared language types;
- protected local state, locked security-journal reconciliation, revocation and
  rebuild-with-new-trust recovery; and
- Compose and Kubernetes-compatible initialisation and readiness definitions.

It does not cover an external-human provider, browser login, managed issuer,
shared or production environment, non-synthetic information, distributed
session state, protected same-environment restore or production security
qualification.

## Evidence summary

| Evidence                             | Result                                                                                                                  | Boundary                                                                         |
| ------------------------------------ | ----------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| Identity and authorisation contracts | Passed: 6 schemas, 12 positive/negative fixtures and 6 compatibility descriptors, within a repository total of 12/24/12 | JSON Schema is canonical; `I-001` has no runtime provider binding                |
| Environment bootstrap                | Passed: independent setups create different environment, trust-domain, signer and fingerprint values                    | Local operating-system randomness and protected filesystem state only            |
| Trust-profile readiness              | Passed: local-synthetic is prominent and hosted/managed declaration fails readiness                                     | No managed trust implementation is claimed                                       |
| Workload authority                   | Passed: the Director workload receives only configured audience and contract actions                                    | Static demonstration configuration, not Kubernetes workload federation           |
| Policy decision                      | Passed: relationship and consent sources, purpose, role, obligations and freshness are enforced                         | Bounded in-process policy, not an external product or general policy language    |
| Demonstration grant                  | Passed: short-lived Ed25519 grant conforms to `I-004`; signed claims cannot be changed                                  | Raw grant is written only to an explicitly protected output file for conformance |
| Environment isolation                | Passed: another environment refuses the grant                                                                           | Local-synthetic profile only                                                     |
| At-most-one session                  | Passed: concurrent processes, duplicate delivery and restart return one session reference                               | One host and one locked state directory; no distributed replicas                 |
| Multi-actor scenario                 | Passed: different synthetic actors establish distinct application sessions                                              | Synthetic actors and data realm only                                             |
| Termination and revocation           | Passed: terminal outcomes are idempotent and trust revocation fails readiness                                           | Local journal state only                                                         |
| Recovery                             | Passed: rebuild creates a new trust domain and invalidates former authority                                             | Protected same-environment restoration is not implemented                        |
| Disclosure boundary                  | Passed: journal scan finds no raw signature, grant identifier, key filename, authorisation header or cookie             | Defence-in-depth field/value checks, not general secret discovery                |
| Deployment definitions               | Passed: Kubernetes renders, Compose YAML parses and a local Minikube deployment reaches identity-required readiness     | Single-node local cluster; `emptyDir` rebuilds trust on Pod replacement          |

## Runtime cases exercised

`pnpm check:m2-runtime` builds the framework host and uses three temporary state
directories to exercise independent bootstrap, declared-profile incompatibility,
workload context resolution, policy-approved issuance, concurrent session
establishment, restart reconciliation, cross-environment and tamper refusal,
expiry, multiple actors and applications, termination, revocation, journal
disclosure checks and rebuild recovery. Temporary grant and state material is
deleted at the end of the run.

The full repository gate is `pnpm check`; `pnpm build` verifies the Rust
workspace and browser packages. CI is configured to build the OCI image and
repeat identity setup, grant issuance and duplicate session establishment in
separate containers sharing one bounded state volume; that hosted result is not
yet part of this local evidence record.

## Live Kubernetes evidence

The accepted baseline was deployed on 26 August 2026 to a local arm64
Minikube `1.38.1` cluster using the QEMU driver, containerd runtime and
Kubernetes `1.35.1`. The images were built directly into Minikube; Docker
Desktop was not required or assumed.

The first deployment attempt exposed a genuine state-boundary fault. `INT-01`
tried to restrict the mode of the Kubernetes-managed volume root, which the
non-root process did not own. The runtime and deployment definitions were
corrected so `IAM-01` and `INT-01` use separate owner-only subdirectories
inside the shared runtime-managed mount. The corrected deployment then
produced this bounded evidence:

- the framework-host and web deployments each reached `1/1` readiness with no
  container restart;
- combined health reported both interaction and identity state as ready and
  displayed the `LOCAL-SYNTHETIC TRUST - ISOLATED SCRATCH USE ONLY` warning;
- the runtime operated as uid and gid `65532`, mounted no Kubernetes service
  account token, and protected the IAM and interaction subdirectories with
  mode `0700`;
- a bounded `I-002` Director workload context issued an owner-only `I-004`
  grant and established an `I-005` synthetic reviewer session;
- repeated delivery of the same grant and establishment operation returned
  one session reference;
- framework-host Pod replacement created a different environment identity and
  trust domain, confirming the declared rebuild recovery; and
- the landing, Workbench, Director and Presentation routes all returned
  successfully through a local port-forward.

This was a manual, single-node development-assurance run. The browser shells
were smoke-tested as static routes; they are not yet connected to the
framework host. No ingress, managed issuer, durable cluster persistence,
multi-replica coordination or production recovery was exercised.

## Acceptance boundary

The founders accepted this evidence on 26 August 2026. M2 is therefore complete
as the local-synthetic development-assurance baseline for the initial
demonstrator.

This acceptance is not evidence that M2 is qualified for hosted, shared,
production-like, production or non-synthetic-data use. Those profiles remain
not-ready until a separately planned managed trust, external-human identity,
workload identity, persistence and recovery binding is approved and produces
its own evidence. Publication through a protected pull request and its hosted
checks remains the release record for this baseline.

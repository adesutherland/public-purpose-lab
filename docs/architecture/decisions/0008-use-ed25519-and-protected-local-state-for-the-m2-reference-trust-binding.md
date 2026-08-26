# ADR-0008: Use Ed25519 and protected local state for the M2 reference trust binding

Status: Accepted implementation baseline
Date: 2026-08-26

## Context

M2 must prove environment-scoped synthetic trust, short-lived signed grants and
cross-environment refusal without selecting a hosted PKI, KMS, HSM or identity
product. The binding must be portable across macOS, Linux, Windows and the
existing container form, and it must preserve the `local-synthetic` versus
`managed` distinction in ADR-0007.

Kubernetes certificate automation is useful but is not by itself an
application-authority design. Transport certificates, workload authentication
and Demonstration Sign-In Grant signatures have different audiences and
lifecycle rules.

## Decision

The M2 reference runtime uses Ed25519 signatures for Demonstration Sign-In
Grants. A signature covers a versioned, length-prefixed UTF-8 field sequence,
not an incidental JSON byte representation. The signed fields include the
environment, trust domain and epoch; grant and establishment identifiers;
synthetic actor; application, audience and surface; Demonstration Session;
roles, purpose and synthetic realm; and validity bounds.

An isolated `local-synthetic` bootstrap:

- obtains a new random environment identity and Ed25519 signing key from the
  operating-system random source;
- creates the private key with exclusive creation and owner-only permissions
  where the platform supports them;
- stores it only beneath the explicitly configured protected IAM state path;
- publishes only the public key, fingerprint, trust-domain identifier, epoch,
  profile and safe readiness information; and
- refuses startup if public state, private state or the declared environment
  identity is inconsistent.

The raw local key file is a scratch-profile mechanism, not a qualified secret
store, certificate authority or recoverable production root. It is not copied
into images, logs, fixtures, ordinary backups or evidence packs.

The signing implementation is behind a narrow signer/verifier boundary. A
`managed` environment is ready only when that boundary is supplied by an
approved managed issuer with accountable custody, rotation, revocation,
recovery and audit. Tests may inject an in-memory managed signer to prove
environment and audience semantics, but that adapter is conformance equipment
and never establishes managed operational readiness.

For workload identity, the reference host creates contexts only for workloads
registered in protected environment configuration and only across its local
process boundary. The planned Kubernetes binding uses a dedicated ServiceAccount
per workload, a projected short-lived token with an explicit audience and
cluster TokenReview or equivalent validation. Default ambient token mounting,
namespace membership and a workload-supplied identifier are not accepted as
identity. Enabling the Kubernetes binding requires its own executable threat
and deployment evidence.

`I-001` remains schema-complete but operationally disabled until an external
identity-provider and subject-mapping ADR is accepted.

## Consequences

- The first operational M2 profile is local-synthetic and single-host.
- Grant signatures are asymmetric, environment-bound and independently
  verifiable without exposing the private key.
- X.509 and cert-manager remain available for transport and managed-issuer
  integrations, but M2 does not misuse a TLS certificate as business
  authority.
- A copied local key or state directory is not a supported environment clone.
- The local binding cannot claim protected recovery, hosted/shared readiness
  or resistance to the host administrator.
- Algorithm agility is explicit; accepting another signing profile requires a
  versioned contract and ADR.

## Validation and review

Evidence must cover independent roots, wrong-environment and wrong-audience
refusal, modified claims, expiry, signer revocation, private-file permissions,
safe diagnostics and absence of keys or raw grants from durable evidence.

Reconsider this decision before enabling an external listener, Kubernetes
workload authentication, managed trust, hardware-backed signing, algorithm
change or protected same-environment recovery.

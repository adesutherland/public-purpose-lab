# ADR-0007: Separate local-synthetic and managed trust profiles

Status: Accepted
Date: 2026-08-26

## Context

The framework supports disposable local demonstrations as well as environments
that may be hosted, shared, production-like or authorised to hold non-synthetic
information. Treating every certificate root as equivalent could allow a
convenient scratch trust mechanism to be mistaken for an appropriate hosted or
real-data security boundary.

The use of cert-manager does not by itself establish the assurance level. A
self-signed or environment-generated CA held as an ordinary cluster secret and
an issuer backed by an accountable managed PKI, KMS or HSM have materially
different custody, recovery and operational properties.

## Decision

Define two trust profiles:

- **`local-synthetic`** is permitted only for a local, isolated, single-user or
  tightly controlled scratch environment using synthetic data or approved
  public material. Environment setup may generate its own synthetic root. The
  profile makes no claim of protection from the host or cluster administrator,
  protected same-environment recovery, production operation or suitability for
  non-synthetic information.
- **`managed`** is required when an environment is hosted for others, shared,
  production-like, production, externally relied upon or authorised to hold
  real personal, organisational or confidential information. Its trust anchor
  and signing custody are provided through an accountable managed trust
  service, such as an organisational PKI or KMS/HSM-backed issuer, with defined
  ownership, rotation, revocation, recovery and audit.

A managed trust service may have a common organisational root, but every
environment receives a distinct environment-scoped issuing or signing identity
and every grant remains bound to its environment and audience. Sharing an
upstream managed anchor never permits cross-environment grant acceptance.

cert-manager may automate certificate lifecycle in either profile. A
cert-manager `SelfSigned` or Secret-backed local CA remains `local-synthetic`;
cert-manager connected to an approved managed issuer may implement the
`managed` profile. cert-manager does not issue application authority, decide
access policy or change a synthetic actor into an external human.

An environment cannot be promoted from `local-synthetic` to `managed` by
changing a label. Promotion creates a new managed trust domain, rotates or
re-establishes environment signers and invalidates or terminates grants and
sessions established under the local-synthetic root. Movement of any retained
information is separately authorised and governed.

The active trust profile is a safe operational fact. Appropriate operations,
readiness, support and evidence views show:

- environment class and trust profile;
- whether the profile is permitted for the environment's hosting, sharing and
  information classification;
- safe issuer or trust-anchor identity, trust epoch and status;
- key-custody and recovery class without key handles or secret locations;
- signer expiry, rotation and revocation readiness; and
- a prominent warning whenever `local-synthetic` is in use.

An environment whose configured trust profile is weaker than its declared
deployment or information profile is not ready for identity or synthetic
sign-in and fails closed.

## Consequences

- Local scratch demonstrations retain a simple environment-generated root.
- Hosted, shared and real-data-capable profiles require a managed issuer even
  when all current actors and data happen to be synthetic.
- The initial M2 implementation can qualify `local-synthetic` independently;
  it cannot claim hosted/shared readiness until a managed binding passes the
  same contract and threat evidence.
- Synthetic identities remain visibly synthetic under both trust profiles and
  cannot acquire production or external-human identity by using a managed
  issuer.
- Trust-profile status becomes part of `IAM-01`, `I-003`, `OPS-01` and evidence
  reporting.
- Selection of cert-manager issuer type, managed PKI, KMS, HSM or provider
  remains a later binding decision.

## Validation and review

Conformance evidence must show that:

- independent local scratch environments create unrelated roots;
- `local-synthetic` is visible in safe operational views;
- a hosted, shared, production-like, production or non-synthetic-data profile
  refuses readiness when configured with `local-synthetic`;
- a managed environment uses an environment-scoped signer and refuses another
  environment's grant even where both chain to one organisational root;
- promotion creates a new trust domain and invalidates former grants and
  sessions; and
- neither visibility nor diagnostics expose usable keys or credentials.

Reconsider this decision if a deployment profile provides stronger
environment-local hardware protection that is demonstrably equivalent to the
managed profile, or if threat evidence shows that the two-profile model is too
coarse.

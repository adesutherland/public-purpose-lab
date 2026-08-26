# Security policy

## Current status

Public Purpose Lab is experimental and has no production service or supported
release. Do not use it with real personal, clinical, donor, employee, or
organisation-confidential data.

## Reporting a vulnerability

Do not open a public issue for a suspected vulnerability, exposed secret, or
privacy weakness. Use GitHub's private vulnerability reporting for this
repository when available. If that channel is not available, contact a founder
privately and share only the minimum information needed to establish a safe
channel.

Please include:

- the affected document, component, or revision;
- the observed and expected behaviour;
- a minimal reproduction using synthetic data;
- the plausible impact; and
- any immediate containment recommendation.

Do not access third-party systems, collect real personal data, or retain secrets
while investigating.

## Security principles

- Treat every external input and AI-generated output as untrusted.
- Use synthetic data by default and minimise all retained information.
- Authenticate people and workloads; authorise each action at its boundary.
- Keep secrets out of code, logs, events, examples, and test fixtures.
- Permit a local synthetic root only in an isolated scratch environment; require
  managed trust for hosted, shared, production-like or non-synthetic-data use,
  and make the active profile operationally visible.
- Preserve provenance, policy decisions, human approvals, and audit evidence.
- Design explicit refusal, abstention, failure, and escalation outcomes.
- Prefer contained, replaceable adapters at external-system boundaries.

## Architecture baseline

The versioned
[framework security model](docs/architecture/security/framework-security-model.md)
defines the current trust zones, principals, authority, information handling
and recovery invariants. The
[M1 threat model](docs/architecture/security/m1-threat-model.md) states the
narrower controls and limitations of the local common-interaction reference
path. The
[M2 threat model](docs/architecture/security/m2-threat-model.md) covers the
local-synthetic identity and session reference path. The framework model is the
accepted framework baseline and the milestone threat models are bounded
implementation baselines; none is a production security or compliance
qualification.

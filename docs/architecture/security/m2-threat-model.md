# M2 identity and synthetic-access threat model

Status: Implementation baseline

Version: 0.1.0

Last reviewed: 26 August 2026

## Scope and claim boundary

This threat model covers the single-host M2 reference path: local-synthetic
environment bootstrap, protected local key and security state, configured
workload and synthetic actor contexts, bounded in-process authorisation,
signed Demonstration Sign-In Grants and backend-only synthetic-session state.

It does not qualify an external listener, browser login, external-human
identity, Kubernetes TokenReview, managed PKI/KMS/HSM, real data, multiple
replicas, protected same-environment recovery or production operation.

## Protected assets

- environment identity, private signing key and trust epoch;
- workload, actor, policy, relationship and consent configuration;
- raw signed grants before bounded consumption;
- replay, revocation and synthetic-session security state;
- accurate readiness, decision and recovery evidence; and
- separation of workload, synthetic-human, external-human and operator
  authority.

## Trust boundaries

The configured IAM state path and local process are protected from ordinary
application and browser access but not from the host administrator. Contract
files, command inputs, scenario data and frontend content are untrusted.
Operational output is safe evidence and never a credential. The in-process
policy adapter is logically separate from IAM validation and session
enforcement even when co-deployed.

## Threats, controls and remaining limits

| ID        | Threat                                                                  | M2 reference control                                                                                                   | Remaining limit                                                                |
| --------- | ----------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| `T-M2-01` | A copied or guessed environment identity accepts another root           | Random environment and trust-domain identifiers; signature, environment, epoch and audience validation                 | Host compromise remains out of scope                                           |
| `T-M2-02` | A local-synthetic root is used for shared, hosted or non-synthetic data | Declared environment/information profile compatibility; visible warning; identity readiness fails closed               | Managed binding is not operationally qualified                                 |
| `T-M2-03` | Private signing material leaks through source, logs or evidence         | Exclusive key creation, restricted path, redacted records and automated disclosure scans                               | Local file is not an HSM or qualified secret store                             |
| `T-M2-04` | Grant claims are altered or substituted                                 | Ed25519 signature over versioned canonical fields; strict environment, audience, application, surface and realm checks | Cross-language signing compatibility requires later independent implementation |
| `T-M2-05` | Expired, premature or revoked trust creates a session                   | Bounded clock tolerance, validity checks, monotonic trust epoch and revocation state                                   | Distributed clock assurance is not qualified                                   |
| `T-M2-06` | Duplicate, concurrent or lost-ack delivery creates multiple sessions    | Exclusive state lock, grant/operation digest, one authoritative outcome and reconciliation                             | One-host binding only                                                          |
| `T-M2-07` | A workload impersonates a synthetic human or vice versa                 | Distinct typed principals; actor and requester both retained; role and action scopes checked independently             | Network workload authentication is deferred                                    |
| `T-M2-08` | The Director or caller self-asserts a relationship or consent           | Only configured synthetic authoritative-source assertions are evaluated; caller fields cannot create source authority  | Initial source is local configuration                                          |
| `T-M2-09` | Deny or indeterminate is overridden                                     | Receiving path accepts only permit and enforced obligations; every other result fails closed                           | Broader component enforcement awaits later slices                              |
| `T-M2-10` | An obligation is ignored after permit                                   | Establishment records and checks the complete required-obligation set before session creation                          | No general obligation language is selected                                     |
| `T-M2-11` | A session reaches the wrong application, surface, role or data realm    | Grant and session are bound to all four plus environment, actor, purpose and Demonstration Session                     | Browser/application session binding is simulated backend-only                  |
| `T-M2-12` | Corrupt or partially restored state silently loses replay protection    | Strict journal parsing; inconsistency makes readiness false; local recovery creates a new trust domain                 | Same-environment recovery is deliberately unsupported                          |
| `T-M2-13` | Revoked or terminated sessions are restored by delayed work             | Monotonic terminal states and current-state checks; delayed establishment cannot replace a terminal result             | Distributed ordering is not qualified                                          |
| `T-M2-14` | Diagnostics enable key, grant, actor or session takeover                | Safe fingerprints, digests, counts and reason classes; no raw grants, keys or reusable session values                  | Authorised host administrator can still inspect process/files                  |
| `T-M2-15` | Reference behaviour is presented as production identity capability      | In-development maturity, local-synthetic warning and explicit unsupported-profile readiness                            | Accurate public communication remains a governance responsibility              |

## Required conformance evidence

The M2 suite covers independent environment roots, cross-environment refusal,
claim modification, expiry, trust-profile incompatibility, principal
separation, authorisation outcomes and obligations, actor/application
multiplicity, duplicate/concurrent/restart establishment, revocation,
termination, corrupt state, rebuild recovery and disclosure scanning.

Every later external transport, identity provider, managed issuer,
authorisation product, application session owner or distributed state binding
extends this threat model before it is accepted.

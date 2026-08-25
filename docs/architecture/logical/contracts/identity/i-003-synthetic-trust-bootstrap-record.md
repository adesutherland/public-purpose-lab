# I-003: Synthetic trust bootstrap record

Status: Working draft

Last reviewed: 25 August 2026

Owner: [`IAM-01`](../../components/iam-01-identity-trust-and-synthetic-session-broker.md)

Semantic type: environment command, query and auditable trust-state fact

## Purpose

`I-003` establishes and records the environment-local trust domain used only
for synthetic demonstration identities. Each environment setup creates a
unique environment identity and a unique synthetic root within that
environment's protected boundary.

The contract records safe public and operational facts about creation,
authorised signers, rotation, revocation and recovery. It never carries private
root or signer material.

## Participants and trust boundary

| Role                   | Participant                                  | Responsibility                                                                                                                |
| ---------------------- | -------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| Bootstrap coordinator  | `PLT-01`                                     | Creates or restores the environment and invokes the protected bootstrap operation exactly once for that environment identity. |
| Protected key boundary | Environment key facility                     | Generates and uses root or subordinate private material without routine export.                                               |
| Trust owner            | `IAM-01`                                     | Verifies and records the trust domain, signer constraints, epochs and status.                                                 |
| Evidence owner         | `AUD-01`                                     | Retains the safe bootstrap, rotation, revocation and recovery evidence.                                                       |
| Operator               | Specifically authorised environment operator | Approves bootstrap, recovery and exceptional rotation actions without receiving private material.                             |

The root's public certificate or equivalent public trust anchor is not secret,
but only components in the same environment accept it for synthetic identity.
No external-human or workload path trusts it for its own purpose.

## Contract variants

| Variant                          | Kind                | Purpose                                                                                                    |
| -------------------------------- | ------------------- | ---------------------------------------------------------------------------------------------------------- |
| `BootstrapSyntheticTrust`        | Environment command | Creates the environment's first synthetic trust domain inside the protected boundary.                      |
| `SyntheticTrustBootstrapped`     | Fact                | Records successful creation without private material.                                                      |
| `GetSyntheticTrustStatus`        | Protected query     | Requests redacted readiness, epoch, signer and recovery status.                                            |
| `SyntheticTrustStatus`           | Query response      | Supplies current safe trust-domain state.                                                                  |
| `UpdateSyntheticSignerState`     | Operator command    | Adds, rotates, restricts or revokes an environment-local signer under approved policy.                     |
| `SyntheticSignerStateChanged`    | Fact                | Records the accepted signer-state change.                                                                  |
| `RecoverSyntheticTrust`          | Recovery command    | Restores the same protected environment trust or explicitly replaces it with a new trust domain.           |
| `SyntheticTrustRecovered`        | Fact                | Records the recovery mode, resulting identity, epoch and invalidation boundary.                            |
| `SyntheticTrustBootstrapRefused` | Command outcome     | Reports duplicate, unsafe, unauthorised or inconsistent bootstrap without creating a second root silently. |

The command set is conceptual. Exact administrative APIs or automation require
an ADR and operator threat model.

## Preconditions and authority

Bootstrap requires:

- a newly allocated, stable environment identifier;
- an authenticated, authorised environment setup operation;
- a protected key boundary appropriate to the deployment profile;
- no existing active or partially established synthetic root for that
  environment, unless using the explicit recovery variant;
- persistent trust-state and replay-state locations with defined recovery; and
- an audit path able to retain the safe result.

Only the bootstrap coordinator may request initial creation. A Scenario
Director, presentation surface, scenario package, application user or ordinary
workload cannot create or replace the trust root.

Operator authority to configure infrastructure does not grant authority to sign
in as a synthetic actor. Signer authority is separately constrained and
recorded.

## Common-envelope requirements

The interaction uses `C-001`, `C-002`, `C-003` and `C-004` and includes:

- command or record identifier and schema version;
- environment and proposed or current trust-domain identifiers;
- bootstrap or recovery operation identifier and idempotency key;
- requesting workload and approving operator references;
- issued and occurred times;
- correlation, causation, purpose and classification;
- current and resulting trust epoch, where applicable; and
- evidence and protected-key-operation references.

No envelope field contains a private key, key-encryption value, recovery secret
or usable signer credential.

## Contract-specific information

The successful bootstrap record contains:

- environment identifier;
- synthetic trust-domain identifier;
- trust profile and version reference, without selecting it here;
- public root trust-anchor representation or stable fingerprint;
- creation time and trust epoch;
- protected key-boundary type and safe protection/attestation reference;
- root exportability status as reported by that boundary;
- initial subordinate or leaf signer public identity, if created;
- signer purpose, permitted synthetic realms, roles and audiences;
- rotation and revocation status references;
- recovery policy and protected-backup presence indicator, not its location or
  secret;
- bootstrap software and environment-profile versions; and
- audit and conformance-evidence references.

A signer-state record contains signer public identity or fingerprint,
certificate or trust-chain reference, permitted purpose, audiences, synthetic
roles, validity, status, trust epoch and change reason.

A recovery record states one of two modes:

- `same-environment-restored` — the same environment identity and trust domain
  were restored from an explicitly protected source with replay and revocation
  continuity; or
- `new-trust-domain-created` — a new environment identity or synthetic root was
  created, and all former signers, grants and sessions are invalid.

## Environment isolation invariants

1. Every fresh environment creates a new environment identity and synthetic
   root during setup.
2. A root is never preloaded in source, image, installer, fixture or scenario
   package.
3. Root and signer private material remains inaccessible outside the
   environment to the strongest protection reasonably available for that
   profile.
4. Another environment never accepts the root as a synthetic trust authority.
5. Copying configuration, data or actor display names does not copy the trust
   domain.
6. A restored clone must either be the authorised continuation of the original
   environment or generate a new environment identity and root before use; it
   cannot operate concurrently as a second environment with the same trust.
7. Synthetic trust never establishes external-human or workload trust.

## Acceptance and refusal

Safe reason classes include:

- `environment-identity-missing`;
- `bootstrap-not-authorised`;
- `trust-already-exists`;
- `partial-bootstrap-detected`;
- `protected-key-boundary-unavailable`;
- `key-protection-insufficient`;
- `persistence-not-ready`;
- `audit-not-ready`;
- `signer-constraint-invalid`;
- `trust-epoch-conflict`;
- `recovery-proof-insufficient`; and
- `unsafe-clone-detected`.

Bootstrap is not reported successful until the protected root, safe public
record, required signer state, persistence and audit fact are consistent. A
warning-only success is not sufficient for a missing security invariant.

## Repetition, ordering and idempotency

- `BootstrapSyntheticTrust` is idempotent only for the same environment and
  bootstrap operation. A duplicate returns the established safe record; it does
  not generate another root.
- A different bootstrap operation against an existing or partial trust domain
  is refused and routed to explicit recovery.
- Signer changes are serialised within the trust domain and state their prior
  and resulting trust epochs.
- A delayed update from an older epoch cannot reactivate a revoked signer or
  supersede a later trust state.
- Recovery is a separately authorised operation and never masquerades as a
  duplicate bootstrap retry.

## Partial failure and recovery

Bootstrap is designed as a reconciled operation rather than assuming one
atomic transaction across key, state and audit facilities. If private material
may have been generated but the public record or audit fact is missing, the
environment enters `synthetic-trust-recovery-required` and cannot issue or
accept synthetic grants.

Recovery must inspect the protected key boundary and persisted trust record
without exporting private material. It then either completes the same
environment trust safely or destroys/revokes the partial trust and creates a
new trust domain under a new operation.

Restoring the same environment requires protection, authorisation and continuity
of root identity, signer status, trust epoch, grant replay state and session
revocation state. If those cannot be demonstrated, recovery creates a new trust
domain and invalidates all earlier grants and sessions.

## Backup boundaries and restore security

Each deployment profile declares one of two supported postures:

- `protected-same-environment-recovery` provides an authorised recovery source
  for the environment identity and synthetic trust material, together with a
  recoverable continuity point for trust epochs, signer status, grant replay,
  revocation and synthetic-session state; or
- `rebuild-with-new-trust-domain` treats the private synthetic root as
  non-recoverable and creates a new environment identity and root after loss.

A protected recovery source is itself security-sensitive. Copying it must not
create a second simultaneously valid environment, export private material
through routine backup interfaces, or allow stale state to reactivate a signer,
grant, actor or session. The chosen key-custody mechanism may use protected
wrapping, escrow or another profile-appropriate method, but requires an ADR and
conformance evidence.

Before a same-environment restore becomes ready, the recovery operation must:

1. establish that this instance is the authorised continuation and contain or
   refuse any competing clone;
2. validate the protected key boundary, environment identity and current trust
   epoch;
3. restore replay, revocation and signer-state continuity to a mutually
   consistent point;
4. reconcile and terminate, or explicitly revalidate, sessions that existed at
   the recovery point;
5. rotate or explicitly re-authorise operational signers according to the
   approved recovery policy; and
6. record the recovery, security fix-up and resulting readiness without
   exposing usable trust material.

Synthetic trust and security-state recovery is separate from the recovery of
uploaded assets, business records, reports, provenance and substantive evidence
owned by other components. Restoring or migrating that data does not restore
the former synthetic trust domain. Conversely, an `I-003` recovery record does
not prove that business or evidence data is complete, current or authorised for
use. The synthetic root is not a business-data or evidence-encryption root;
replacing it must not by itself disclose evidence or make retained evidence
unrecoverable.

External identity-provider credentials are never included. Local mappings from
external subjects and authority configuration may have their own protected
configuration recovery, but remain separate from synthetic root material.
Future environments authorised to hold real evidence require separately
governed data-encryption-key, privacy, retention, backup and restore decisions.

## Audit, retention and provenance

Audit retains:

- bootstrap, signer-change and recovery operation identifiers;
- environment, trust-domain and trust-epoch identifiers;
- public root and signer fingerprints;
- authorised workloads and operators;
- safe key-protection and non-exportability evidence;
- software, configuration and recovery-policy versions;
- accepted, refused or recovery-required outcomes and reason codes;
- old and new trust status without private material; and
- conformance-evidence references.

The trust record lasts for the life of the environment plus the agreed evidence
period. Revoked and superseded signer references remain available for historical
validation and incident reconstruction.

## Analytical use

Permitted measures include bootstrap success or refusal, trust age, signer
rotation, revocation, recovery mode and time spent not ready. The public trust
fingerprint need not be a routine analytical dimension, and private-key or
recovery information is prohibited.

Analytics cannot establish trust or determine current signer acceptance.

## Operations and observability

Readiness covers:

- protected key-boundary availability;
- consistency of environment identity, trust record and trust epoch;
- signer validity and revocation data;
- replay and session-state persistence readiness;
- clock health;
- safe backup or declared no-backup recovery posture; and
- audit path availability.

Diagnostics reveal only safe identifiers and state. They must not reveal key
handles if those handles can assist unauthorised use, recovery-secret metadata,
or protected backup locations.

## Deployment considerations

The same isolation invariant applies to local macOS, Linux, Windows, portable
demonstration and hosted profiles. The strongest reasonably available
environment-local protection may differ, and its limitations must be explicit
in that profile's conformance evidence.

Container images are immutable software artifacts, not an environment trust
boundary. Root generation occurs during environment bootstrap after deployment,
and persisted private material is held outside the image. A hosted replica set
belongs to one environment trust domain; independent hosted environments do
not share it.

## Versioning and compatibility

Each variant declares its schema version through `C-006`. New optional safe
evidence references may be compatible. Changes to environment identity,
trust-domain construction, root meaning, signer constraints, epochs,
non-exportability, clone handling or recovery modes are breaking.

Consumers refuse unsupported trust records rather than assuming a public key is
sufficient. Deprecation preserves historical fingerprints and the ability to
interpret grants and session outcomes issued under earlier supported epochs.

## Transport-neutral examples

An accepted example is: first setup of environment `demo-a` invokes one
bootstrap operation inside its protected boundary and records environment
identity `demo-a`, a newly generated root fingerprint, epoch one and a
restricted demonstration signer. A fresh `demo-b` setup records a different
identity and root even if it loads the same scenarios and actor names.

A negative example is: a restored copy presents `demo-a`'s trust record while
claiming to be a new environment. Bootstrap refuses the unsafe clone; recovery
must prove it is the one authorised continuation or create a new environment
identity and root.

## Threat considerations

The threat model must address:

- root extraction, export or unintended backup;
- weak entropy or predictable environment identity;
- image, fixture or scenario package containing pre-generated trust material;
- environment cloning and simultaneous use of restored copies;
- unauthorised bootstrap, signer addition or root replacement;
- partial bootstrap and rollback to an unsafe epoch;
- stale revocation information;
- compromised platform operator or bootstrap workload;
- abuse of root directly as a routine signer;
- recovery data disclosure or loss; and
- cross-use of synthetic trust for human or workload access.

## Conformance evidence

Evidence must show that:

1. two independently bootstrapped environments have different environment
   identities, roots and trust-domain identifiers;
2. neither image, source, installer, fixture nor scenario package contains
   private or pre-generated root material;
3. private material cannot be retrieved through documented application,
   support, backup or diagnostic interfaces;
4. another environment refuses the root as a synthetic authority;
5. duplicate bootstrap returns the existing record and does not generate a
   second root;
6. partial bootstrap fails closed and is resolved only through explicit
   recovery;
7. signer constraints prevent human, workload, excessive-role and wrong-audience
   issuance;
8. rotation and revocation advance trust state and older delayed updates cannot
   reverse it;
9. same-environment recovery preserves replay and revocation continuity;
10. new-trust-domain recovery invalidates every former grant and session;
11. a same-environment restore performs the required clone, replay, revocation,
    session and signer security fix-up before readiness;
12. synthetic trust recovery remains separate from evidence and business-data
    recovery; and
13. audit reconstructs bootstrap and recovery without exposing usable key
    material.

Each supported deployment profile requires its own key-protection and recovery
evidence.

## Open ADR decisions

- environment identity generation and persistence;
- certificate or equivalent trust profile and signing algorithm;
- root, subordinate and leaf signer hierarchy;
- key generation, non-exportability and protected use by deployment profile;
- root use restrictions, rotation, revocation and trust epochs;
- protected backup, restore, clone detection and no-backup options;
- separation and restore ordering for synthetic trust, security state, local
  identity mappings, evidence and business data;
- post-restore signer rotation, session termination and revalidation policy;
- bootstrap transaction, reconciliation and destruction of partial material;
- signer constraint and approval representation; and
- public trust-record distribution and retention.

No decision may introduce a shared cross-environment synthetic root or package
private trust material outside its environment.

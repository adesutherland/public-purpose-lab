# D-001: Scenario package

Status: Accepted M3.1 logical baseline; schema and implementation pending

Version: 0.1.0

Last reviewed: 27 August 2026

Governing decision:
[ADR-0011](../../../decisions/0011-establish-the-m3-scenario-control-invariants.md)

Owner: [`CTL-01`](../../components/ctl-01-scenario-director.md), with every
referenced action still owned by its receiving component

Semantic type: immutable, declarative scenario definition and admission result

Canonical schema: Not selected in M3.1

## Purpose

`D-001` describes one repeatable synthetic assurance scenario independently of
its runtime, transport, browser routes and deployment topology. It declares the
actors, surface roles, participating capabilities, stages, control actions,
business-command references, semantic cues, readiness conditions, checkpoints,
reset scope, controlled-time needs, bounded faults and expected evidence.

A package is a plan and provenance record. Admission does not authenticate a
principal, authorise a business action, establish a session, prove readiness or
assert that an expected outcome occurred.

## Participants and authority

| Role              | Participant                                | Responsibility                                                                                               |
| ----------------- | ------------------------------------------ | ------------------------------------------------------------------------------------------------------------ |
| Package publisher | Identified scenario owner                  | Versions the definition, records provenance and proposes its permitted environment profile.                  |
| Admission owner   | `CTL-01` owner and runtime                 | Reviews policy and validates the package, digest, compatibility and declared capabilities.                   |
| Capability owner  | Each referenced component or surface owner | Publishes supported semantic capabilities and independently accepts or refuses an invocation.                |
| Presenter         | Authorised external or synthetic human     | Operates an admitted package within a Demonstration Session; cannot alter authority through package content. |
| Platform owner    | `PLT-01`                                   | Supplies approved package and fixture distribution without becoming scenario or business authority.          |

Package provenance identifies who published the definition. It does not grant
that publisher authority over a receiving component. `C-002`, current identity,
policy and component state govern each protected operation when invoked.

## Contract variants

| Variant                           | Kind                             | Purpose                                                                                        |
| --------------------------------- | -------------------------------- | ---------------------------------------------------------------------------------------------- |
| `ScenarioPackage`                 | Immutable definition             | Declares one exact package version and its requirements.                                       |
| `ScenarioPackageAdmissionRequest` | Command                          | Requests admission into a named environment and supported profile.                             |
| `ScenarioPackageAdmissionOutcome` | `C-003` outcome plus safe detail | Reports accepted, refused, duplicate, expired or failed admission without beginning a session. |

Admission is separate from `D-002` session creation and start. A package can be
admitted while its current runtime dependencies are not ready.

## Package identity and integrity

Every package declares:

- stable package identifier and immutable semantic version;
- human-readable title, purpose and bounded description;
- publisher and accountable owner references;
- creation and release times and lifecycle status;
- content digest with algorithm and canonical-content profile;
- provenance and rights references for the package and every fixture;
- required contract and capability versions using `C-005` and `C-006`;
- permitted environment, trust and information profiles; and
- predecessor, replacement or deprecation references where applicable.

Changing any semantic content creates a new version and digest. A display-name,
formatting or ordering change may still be semantic when it alters a cue,
checkpoint, authority expectation or evidence interpretation.

A digest supports integrity comparison but does not prove authorship or approve
the package. Signing, canonical serialisation, repository and distribution
mechanisms remain implementation decisions for a later ADR.

## Declarative content model

### Scenario metadata and limits

The package states:

- scenario purpose, intended audience and assurance questions;
- maturity and explicit non-claims;
- synthetic-only information classification and permitted public material;
- supported local, portable or hosted profiles without implying qualification;
- maximum duration, operation counts and other bounded resource expectations;
- retention and reset expectations; and
- known limitations and conditions that require presenter explanation.

### Actors and authority expectations

Each named scenario actor includes a stable package-local role reference,
display name, principal type, intended application roles and purpose. Runtime
binding resolves it to an environment-scoped identity through `IAM-01`.

The package may request a synthetic actor or identify a presenter role. It
cannot create the principal, relationship, consent, professional authority,
workload identity or application session. Actor display names may repeat across
environments; runtime principals cannot.

### Components, applications and surfaces

The package identifies required and optional:

- logical components and semantic capabilities;
- contract families and supported version ranges;
- application roles and data realms;
- presentation surface roles and semantic view capabilities; and
- readiness, evidence and reset capabilities.

These are logical identifiers. Internal service names, broker topics, browser
URLs, routes, database keys and secret locations are prohibited.

### Fixtures

A fixture declaration records an opaque fixture identifier, immutable digest,
media and schema type, provenance, rights, synthetic/public classification,
owning component, load purpose and expected reset owner.

Fixture content is not embedded when it would duplicate substantive source or
security material. A reference is not authority to retrieve or use the fixture;
the owner applies admission, rights, classification and size controls.

M3 uses synthetic fixtures only. Real personal, clinical, donor, employee,
client-confidential or otherwise restricted content is refused unless separate
future authority and governance explicitly replace this limitation.

### Stages and steps

Stages give a human-readable structure to the scenario. Each step has:

- stable identifier, description and optional presenter guidance;
- explicit prerequisites and applicable session states;
- one or more separately typed actions or observations;
- authority, target, purpose and expiry expectations for each command;
- correlation and evidence expectations;
- success, refusal, uncertainty and safe recovery paths; and
- optional next-step conditions based on `D-004` observations.

A step never treats a presentation cue as a business command. Business change
is represented by a separately authorised command owned and validated by its
target. A wait or checkpoint observes declared facts and cannot directly mutate
state.

### Presentation intent

The package may identify a semantic view and bounded context needed at a stage.
It contains no browser route, URL, credential or session value. `P-001` to
`P-004`, defined in M3.2, bind that intent to an authenticated registered
surface and report a presentation-only outcome.

### Reset, logical time and faults

The package declares only named `D-003` capabilities already offered by their
owners:

- component-owned reset targets, prerequisites and required/optional status;
- initial logical-time context and supported logical-time operations; and
- allow-listed adverse-case profiles, maximum duration and required cleanup.

It cannot contain SQL, shell commands, infrastructure patches, executable
scripts, arbitrary fault parameters or instructions to rotate, delete or export
trust and security state.

### Readiness, checkpoints and evidence

Each readiness condition or checkpoint names:

- the semantic claim being evaluated;
- required source component, contract, fact or safe evidence kind;
- allowed version, freshness and correlation rules;
- expected result and evaluation rule;
- whether absence is failure, pending or not-evaluable; and
- classification, retention and audience for the evaluation record.

Software health, scenario readiness, presentation progress and business
completion are different claim types and cannot satisfy one another implicitly.

## Prohibited content

A package is refused if it contains or requires:

- a private key, password, API key, token, cookie, signed grant, session secret
  or reusable credential;
- a browser URL, internal route, host path, broker topic, private endpoint,
  table name or storage key used as the meaning of an action;
- an external-human credential or a self-asserted relationship, consent, legal
  basis or professional authority;
- arbitrary executable code, shell, SQL, infrastructure or network-control
  instructions;
- an undeclared side effect hidden in a checkpoint, query, cue or fixture load;
- real or confidential data under the M3 information profile; or
- a claim of compliance, production readiness or endorsement unsupported by
  separate evidence and governance.

Public provenance or documentation may eventually use governed external
references, but they are not runtime routes and are outside the first assurance
package.

## Admission and refusal

Before acceptance, `CTL-01` verifies structure, exact package digest, supported
contracts, profile compatibility, fixture provenance, bounded resources,
declared principals and capabilities, separation of action types, and the
prohibited-content rules.

Safe refusal reasons include:

- `package-version-unsupported`;
- `package-integrity-unconfirmed`;
- `publisher-or-provenance-unaccepted`;
- `environment-profile-incompatible`;
- `information-profile-incompatible`;
- `required-contract-unsupported`;
- `required-capability-unsupported`;
- `fixture-provenance-or-rights-unaccepted`;
- `hidden-route-or-secret-material`;
- `executable-content-not-permitted`;
- `authority-boundary-invalid`; and
- `resource-bound-exceeded`.

Admission errors do not reveal secret configuration or enumerate unavailable
principals and surfaces to an unauthorised caller.

## Common-envelope requirements

Admission requests use `C-001` with `C-002` authority, purpose, environment,
target, classification, correlation, expiry and idempotency. Outcomes use
`C-003` and link retained admission evidence through `C-004`.

The package definition itself is content, not an authority-bearing envelope.
Transport metadata cannot silently replace or modify its canonical identity,
version or digest.

## Idempotency, ordering and lifecycle

- Admission is idempotent for one environment, package version, digest and
  admission-policy version.
- Reusing an admission idempotency key for different content is refused.
- A superseding package version does not alter sessions already bound to an
  earlier immutable version.
- Package withdrawal prevents new admission or session creation according to
  policy; it does not rewrite historical evidence.
- Current component readiness and authority are re-evaluated at execution time;
  an old admission outcome is not a permanent permit.

## Audit, privacy and analytical use

Admission evidence records package identity, version, digest, publisher and
policy references, environment and profile, requirement versions, decision,
safe reason, time, correlation and evidence references. It excludes package
secrets because packages are not allowed to contain any, and it does not copy
fixture content into routine logs or analytics.

Permitted analytics include admission, compatibility and capability failures by
safe class. Analytics cannot admit a package or change its lifecycle.

## Versioning and compatibility

`D-001` follows `C-006`. Changes to authority separation, action meaning,
checkpoint evaluation, prohibited content, reset scope, logical-time safety,
fault containment or information classification are breaking even if represented
by optional fields.

An implementation supports only versions it explicitly declares. Unknown
action kinds and authority-bearing fields are refused. Deprecation preserves
the ability to interpret retained session evidence.

## Transport-neutral examples

An accepted example defines a charity systems-discovery assurance run with one
Director Console, Workbench and Presentation surface roles, two named synthetic
actors, semantic views, explicit lifecycle controls, separately authorised
synthetic business actions, observable checkpoints, deterministic reset and
duplicate/out-of-order adverse cases.

A refused example embeds `/workbench/review/42?session=...` as a cue and a shell
command to clear a database. Even if it would work locally, it couples meaning
to a route, exposes session context and bypasses component-owned reset.

## Conformance evidence

Evidence must demonstrate:

1. immutable version and digest handling, including changed-content conflict;
2. refusal of unsupported contracts, capabilities and environment profiles;
3. refusal of secrets, routes, executable content and undeclared side effects;
4. synthetic actor names cannot become runtime principals without M2 binding;
5. business commands, cues, queries and checkpoints remain distinct;
6. package admission cannot override a receiving component's current decision;
7. withdrawn or superseded packages do not rewrite prior session evidence;
8. fixture provenance, rights, classification and reset ownership are checked;
9. the same admitted semantics can support live and automated assurance; and
10. logs, outcomes and analytics do not copy fixture or security-sensitive
    content.

## Open implementation decisions

M3.1 does not select the package serialisation, schema, canonicalisation,
signature, repository, distribution API, cache, admission store or executable
rules engine. These bindings follow approval of the logical contract.

The first contract remains declarative. If executable scenario rules later
provide demonstrated value, cREXX is the preferred first assessment and the
bounded execution surface requires an ADR and separate threat analysis.

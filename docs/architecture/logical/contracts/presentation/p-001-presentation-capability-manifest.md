# P-001: Presentation capability manifest

Status: Accepted M3.2 logical baseline; schema and implementation pending

Version: 0.1.0

Last reviewed: 27 August 2026

Owner: The application owner publishing a presentation-capable release;
admission owned by
[`CTL-02`](../../components/ctl-02-presentation-gateway-and-screen-registry.md)

Semantic type: immutable capability description and admission outcome

Canonical schema: Not selected in M3.2

## Purpose

`P-001` declares which semantic presentation views an application release can
resolve, which bounded context each view accepts and which presentation
constraints it observes. It specialises `C-005` without exposing browser
routes, URLs or implementation details.

A manifest is descriptive. It does not authenticate a live surface, authorise
a cue, establish an application session or prove that a view was shown.

## Participants and authority

| Role               | Participant                      | Responsibility                                                                                      |
| ------------------ | -------------------------------- | --------------------------------------------------------------------------------------------------- |
| Manifest publisher | Application owner and build      | Publishes an immutable, attributable description of capabilities actually implemented by a release. |
| Admission owner    | `CTL-02`                         | Validates identity, compatibility, profile and policy before accepting a manifest.                  |
| Requirement source | `D-001` package through `CTL-01` | Declares required semantic views and versions without prescribing their routes.                     |
| Resolver           | `UX-04` application surface      | Maps an accepted semantic view to local frontend behaviour.                                         |

Manifest admission cannot grant presenter, workload, synthetic-user or
business authority.

## Contract variants

| Variant                                 | Kind                 | Purpose                                                                           |
| --------------------------------------- | -------------------- | --------------------------------------------------------------------------------- |
| `PresentationCapabilityManifest`        | Immutable definition | Declares the views and constraints implemented by one application release.        |
| `PresentationManifestAdmissionRequest`  | Command              | Requests admission of an exact manifest into one environment.                     |
| `PresentationManifestAdmissionOutcome`  | `C-003` outcome      | Reports accepted, refused, duplicate or failed admission using safe reason codes. |
| `PresentationManifestWithdrawalRequest` | Command              | Prevents new registrations using a withdrawn manifest.                            |

Withdrawal does not rewrite historic registrations or evidence. Current
registrations follow explicit expiry or revocation policy.

## Manifest identity and content

Each manifest contains:

- stable manifest identifier, immutable version and content digest;
- publisher, application, application release and build-provenance references;
- supported `P-001` to `P-004`, common-envelope and profile versions;
- supported local-container, Minikube or hosted profiles without overstating
  their evidence maturity;
- generated, valid-from, deprecation and replacement information;
- semantic view declarations; and
- conformance-evidence and known-limitation references.

Each semantic view declaration contains:

- a stable semantic view identifier and version range;
- a human-readable purpose and intended audience class;
- allowed context field names, types, size bounds and classification;
- required and optional context references;
- accessibility, reduced-motion and focus-management expectations;
- safe empty, loading, refusal and failure states;
- maximum supported update rate or other bounded rendering constraints; and
- compatibility and deprecation information.

The manifest contains no URL, path, host, query string, fragment, DOM selector,
window identifier, event-broker subject, cookie, token or session value. The
application owns the local mapping from semantic view to implementation.

## Context and data minimisation

A view accepts only the context it needs to render its declared meaning. The
first profile permits synthetic, public or privacy-minimised references only.
Substantive source content, evidence and application records remain with their
owners and are retrieved through separately authorised application paths.

A manifest cannot make a field safe merely by declaring it. Environment and
information-profile policy may impose tighter classifications, sizes or
audiences. Unknown or broader context is refused rather than ignored where it
could change meaning or disclosure.

## Admission and compatibility

`CTL-02` admits the manifest only when:

1. publisher, application and release provenance are accepted;
2. digest and supported contract versions are exact and attributable;
3. every semantic identifier and context shape is unambiguous and bounded;
4. the environment and information profiles are compatible;
5. prohibited route and credential fields are absent;
6. required accessibility and safe-failure declarations are present; and
7. stated maturity is supported by evidence references.

Safe refusal classes include `manifest-integrity-unconfirmed`,
`publisher-unaccepted`, `application-release-mismatch`,
`contract-version-unsupported`, `semantic-view-invalid`,
`context-profile-unsafe`, `route-or-credential-present`,
`accessibility-declaration-missing` and `resource-bound-exceeded`.

Compatibility follows `C-006`. Adding an optional view may be compatible;
removing or changing the meaning, required context, classification or failure
behaviour of a supported view is not silently compatible.

## Idempotency, evidence and privacy

Admission is idempotent for environment, application release, manifest version
and digest. Reuse of an admission identity for different content is refused.

Evidence records safe identity, version, digest, provenance, supported
capability identifiers, profile, decision, reason, time and evidence
references. It does not copy application code, internal mappings or credential
material.

## Conformance evidence required

Evidence must show that an admitted release:

- resolves every claimed semantic view and refuses every unsupported view;
- accepts only declared, bounded context and handles absent optional context;
- provides declared loading, empty, refusal and failure states;
- preserves required accessibility and focus behaviour;
- does not use cue content as a URL or business instruction;
- treats incompatible manifest changes as a new version; and
- leaks no route, credential, raw session or substantive protected content
  through its manifest or diagnostics.

## Current limitation

This is an accepted logical specification. The canonical JSON Schema, manifest
signing or distribution mechanism and executable frontend conformance harness
are pending.

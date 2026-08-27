# ADR-0017: Use canonical JSON and repository provenance for M3 scenario packages

Status: Accepted

Date: 2026-08-27

## Context

`D-001` requires an immutable, declarative package with an exact content digest,
closed compatibility rules, attributable provenance and no executable content,
route or credential. M3.3 needs one package that is identical in native,
container, Minikube and private hosted checks.

The first slice does not need runtime upload, an external package registry,
third-party publishers or general signature verification. Introducing those
mechanisms now would expand the content, identity and recovery attack surface
before one package has demonstrated value.

Plain JSON object ordering and number representation are not sufficient for a
portable digest. A compressed archive also introduces path, extraction and
duplicate-file risks that the first runtime does not need.

## Decision

Represent M3 scenario packages as a closed directory bundle containing:

- `manifest.json`, which identifies the package/version and records the
  canonical scenario and fixture digests;
- `scenario.json`, which conforms to the canonical `D-001` JSON Schema; and
- optional small synthetic fixture files listed exactly in the manifest.

Use JSON Schema Draft 2020-12 and refuse unknown object fields, duplicate JSON
property names, non-I-JSON content, unlisted files, path traversal, links and
content outside the declared size/media/schema bounds.

Canonicalise each JSON document using RFC 8785 JSON Canonicalization Scheme and
calculate SHA-256 over its UTF-8 canonical bytes. The package digest is the
SHA-256 of the canonical manifest, which contains the scenario and fixture
digests but no self-referential package-digest field. The admitted `D-001`
record binds package identity, semantic version, package digest, source
revision and application image digest.

Use strings for timestamps, durations and identifiers where alternative JSON
number interpretations could alter meaning or digest. Numerical fields remain
within the JCS/I-JSON interoperable range and are covered by cross-language
canonicalisation fixtures.

For M3.3, build the one founder-reviewed `presentation-control-assurance`
package into the immutable application image as read-only files. The runtime
does not fetch, upload, unpack or modify packages. Repository review, the exact
source revision, CI evidence and the immutable image digest provide the first
publisher/provenance boundary.

This is integrity comparison and controlled provenance, not a claim of
third-party authorship or non-repudiation. External package distribution,
runtime upload and a package signature profile require a later ADR and are
mandatory before untrusted publishers or a shared package catalogue are
accepted.

## Alternatives considered

- **YAML authoring and runtime parsing:** readable, but implicit types, aliases
  and parser variation add ambiguity to the first integrity boundary. A future
  authoring tool may generate canonical JSON without making YAML the runtime
  contract.
- **Sign every package in M3.3:** potentially useful, but key ownership,
  publisher admission, revocation and distribution are not yet defined.
  Signing an ungoverned package would add ceremony without resolving authority.
- **OCI artifact per package:** a credible future distribution binding, but it
  introduces registry client, authentication and extraction work before the
  first package loader is proven.
- **Package embedded only as Rust data:** simple to execute but obscures the
  portable public contract and prevents independent schema/digest checks.
- **ZIP or tar bundle:** familiar distribution, but archive extraction and
  path handling are unnecessary for the image-bundled M3.3 package.

## Consequences

- One human-inspectable package representation can be validated consistently
  by repository tooling and the Rust runtime.
- Formatting and object order do not alter the digest, while semantic content
  does.
- Canonicalisation becomes security-sensitive code and needs official vectors,
  duplicate-key tests and cross-runtime evidence.
- Updating any package or fixture produces a new digest and application image;
  a running Demonstration Session remains bound to its admitted version.
- M3.3 cannot upload or select arbitrary packages at runtime. That is a
  deliberate scope boundary rather than a missing user feature.
- Image provenance is sufficient only for the reviewed first-party M3.3 path;
  package signing and external distribution remain open.

## Validation and review

Evidence must show:

- identical canonical bytes and SHA-256 results across supported development
  platforms and the runtime image;
- acceptance of canonical examples and refusal of duplicate keys, unknown
  fields, invalid Unicode/numbers, changed digests, unlisted files, links and
  path traversal;
- refusal of URL, route, broker subject, credential-like and executable
  package content;
- identical package, source and image digest evidence in native, container,
  Minikube and private hosted checks; and
- no runtime write, upload, fetch or archive-extraction path.

Review this choice before packages can be supplied outside the reviewed
repository, selected dynamically, signed by more than one publisher or stored
independently of application releases.

## Reference material

- [RFC 8785: JSON Canonicalization Scheme](https://www.rfc-editor.org/info/rfc8785/)
- [JSON Schema specification](https://json-schema.org/specification)

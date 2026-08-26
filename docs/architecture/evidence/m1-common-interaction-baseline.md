# M1 evidence: common interaction and runtime baseline

Status: Accepted M1 development-assurance baseline

Evidence date: 26 August 2026

## Scope

This record covers the current M1 development-assurance profile:

- framework security model and M1 threat model;
- `C-001` to `C-006` working-baseline schemas and compatibility descriptors;
- shared Rust and TypeScript contract consumption;
- the local `INT-01` conformance-probe command/outcome path;
- single-host append-journal idempotency, restart and safe diagnostics; and
- container and Kubernetes-compatible packaging definitions.

It does not cover authenticated identity, an external API, event broker,
distributed delivery, business persistence, production audit retention or
production security qualification.

## Evidence summary

| Evidence                           | Result                                                                                                             | Boundary                                                                                                  |
| ---------------------------------- | ------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------- |
| Architecture catalogue             | Passed: 21 components and 40 contract families                                                                     | Catalogue consistency, not implementation of every component                                              |
| Common contract validation         | Passed: 6 JSON Schema 2020-12 schemas, 12 positive/negative fixtures and 6 compatibility descriptors               | Contract semantics accepted; implemented consumers remain limited to the stated profile                   |
| Rust contract consumption          | Passed: canonical command, outcome, manifest and descriptor deserialisation                                        | Hand-maintained types checked against examples; schema remains canonical                                  |
| TypeScript contract consumption    | Passed: strict types and 4 tests                                                                                   | No live browser transport or authentication                                                               |
| `INT-01` unit evidence             | Passed: 11 tests                                                                                                   | macOS arm64 native development profile                                                                    |
| First delivery                     | Passed: one `accepted` `C-003` outcome with `C-004` evidence reference                                             | Local assurance operation only                                                                            |
| Duplicate and restart              | Passed: separate process returns `duplicate` linked to the original outcome; one operation applied                 | One host and one locked state directory                                                                   |
| Concurrency                        | Passed: 8 concurrent deliveries produce one acceptance and 7 duplicates                                            | Single-host standard-library file lock; Windows was not exercised and no distributed replicas are claimed |
| Conflict, expiry and compatibility | Passed: changed semantic content, expired requests and unsupported versions are safely refused                     | One implemented conformance capability                                                                    |
| Authority/environment boundary     | Passed: another environment and invalid principal/authority constraints refuse                                     | Structure and consistency only; M1 does not authenticate metadata                                         |
| Disclosure boundary                | Passed: journal contains no payload, raw idempotency key, authentication reference, issuer or principal identifier | Field-name scanning is defence in depth, not general secret detection                                     |
| Corrupt state                      | Passed: invalid history makes interaction readiness false                                                          | No automatic repair or qualified backup/restore                                                           |
| Full source gate                   | Passed: formatting, lint, tests, links, schemas, types and runtime check                                           | Local checkout with temporary Rust toolchain                                                              |
| Full build                         | Passed: Rust workspace and all browser/package builds                                                              | Native build only                                                                                         |
| Kubernetes rendering               | Passed: base renders with non-root security context, read-only root and declared writable state mount              | `emptyDir` survives container restart, not Pod replacement                                                |
| OCI container execution            | Passed in hosted Linux CI: image build and accepted/duplicate test across two containers using one volume          | One container host and volume; no distributed, high-availability or network-filesystem claim              |

## Runtime cases exercised

The native suite demonstrates:

- exact schema/example consumption;
- accepted first delivery;
- sequential, concurrent, metadata-only and post-restart duplicates;
- semantic idempotency conflict;
- retryable early delivery followed by one in-window acceptance;
- expired and unsupported-version refusal;
- cross-environment authority refusal;
- sensitive-field refusal without journal disclosure; and
- fail-closed processing and readiness for corrupt or unavailable state.

The executable repository gate is `pnpm check`. It includes the Rust and
TypeScript suites and starts the host in separate processes through
`tools/check-m1-runtime.mjs`. `pnpm build` verifies all native packages and
browser builds. CI adds the container restart test defined in
`.github/workflows/ci.yml`.

## Acceptance decision

The founders accepted M1 on 26 August 2026:

1. the framework security model, threat model and common contracts were
   reviewed and accepted;
2. ADRs `0004` to `0006` were accepted, including the clarified compatibility
   direction and the bounded local-journal limitation;
3. the authorisation review finding was resolved by recording the
   externalisable `AUT-01` policy-decision boundary and planned `AZ-001`
   contract without prematurely selecting a product; and
4. GitHub Actions run
   [32872238000](https://github.com/adesutherland/public-purpose-lab/actions/runs/32872238000)
   passed the source, Linux build and container lifecycle gates for implementation
   commit `2966c42`, merged through pull request `#3` as `e984df1`.

M1 is therefore complete as an accepted development-assurance baseline.
`INT-01` remains `in-development`; milestone acceptance does not promote the
component to demonstrated or production maturity.

Windows native behaviour, multi-replica delivery, durable hosted persistence,
tamper-evident evidence and production recovery are not qualified by M1. A
later baseline may revise the mechanism, but it must preserve or explicitly
reconsider the security and interaction invariants with new evidence.

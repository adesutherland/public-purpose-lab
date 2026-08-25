# Backend workspace

The backend is a Rust workspace for the framework host and for logical
component packages that have reached implementation. A package boundary does
not imply a separately deployed service.

The workspace contains:

- `apps/framework-host`: the local command/outcome adapter, component
  description, capability manifest and safe health boundary;
- `crates/contracts`: shared Rust representations of the canonical common
  schemas;
- `crates/core`: shared identifiers and component descriptors; and
- `components/iam-01`: the package boundary for IAM-01, intentionally limited
  to its descriptor and contract inventory; and
- `components/int-01`: the in-development M1 contract, idempotency, journal and
  outcome reference implementation.

The M1 implementation has no listener and does not authenticate authority
metadata. Its append journal is qualified only as single-host assurance state;
it is not an event transport, business store or production audit system. No
certificate generation, synthetic sign-in or external identity integration is
implemented. cREXX remains excluded while its future `RUL-01` boundary is
designed.

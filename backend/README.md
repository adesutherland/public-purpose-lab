# Backend workspace

The backend is a Rust workspace for the framework host and for logical
component packages that have reached implementation. A package boundary does
not imply a separately deployed service.

The initial workspace contains:

- `apps/framework-host`: a dependency-light executable proving the common host
  build, description and health-check boundary;
- `crates/core`: shared identifiers and component descriptors; and
- `components/iam-01`: the package boundary for IAM-01, intentionally limited
  to its descriptor and contract inventory until the trust contracts are
  approved.

No certificate generation, synthetic sign-in, event transport, persistent
storage or external identity integration is implemented by this skeleton.
cREXX is also excluded from the initial build while its future RUL-01 boundary
remains in the architecture catalogue.

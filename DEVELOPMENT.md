# Development guide

## Current status

This repository contains a buildable framework with M1 common-interaction and
M2 local-synthetic identity reference paths in development. It proves common
packaging, presentation surfaces, architecture catalogues, strict contracts,
safe command outcomes, environment-scoped demonstration trust and local
idempotency/restart behaviour.

Both adapters are local assurance profiles with no network listener. M2 can
establish bounded synthetic session state, but it is not a browser login,
external identity provider, managed issuer, production identity service or
distributed session store. The repository does not yet provide workflow,
business persistence, retrieval, reporting, analytics or cREXX execution.

## Prerequisites

- Node.js 24 or later;
- pnpm 11.19 or later; and
- Rust 1.98 with `rustfmt` and `clippy`.

Docker or another OCI-compatible container runtime is optional. Kubernetes is
required only to render or apply the deployment example.

## Build and check

```sh
pnpm install --frozen-lockfile
pnpm check
pnpm build
```

Run a frontend locally with one of:

```sh
pnpm dev:workbench
pnpm dev:director
pnpm dev:presentation
```

Inspect the backend boundary with:

```sh
cargo run --bin ppl-framework-host -- describe
cargo run --bin ppl-framework-host -- healthcheck
cargo run --bin ppl-framework-host -- manifest
```

Exercise the M1 command/outcome path with a temporary state directory and a
fixed assurance clock:

```sh
PPL_M1_STATE="$(mktemp -d)"
cargo run --bin ppl-framework-host -- process \
  --state-dir "$PPL_M1_STATE" \
  --environment-id env-local-001 \
  --now 2030-08-25T12:01:00Z \
  contracts/common/examples/c-001-m1-conformance-command.json
```

Repeat the command to observe a correlated duplicate outcome without a second
operation. The configured state directory contains a privacy-minimised local
journal; do not point it at a shared or authoritative data location. The
`serve` command qualifies process and container lifecycle only and exposes no
network service.

Exercise the M2 local-synthetic path with an independently created temporary
environment:

```sh
PPL_M2_STATE="$(mktemp -d)"
cargo run --bin ppl-framework-host -- iam-configure-demo \
  --state-dir "$PPL_M2_STATE"
cargo run --bin ppl-framework-host -- iam-health \
  --state-dir "$PPL_M2_STATE"
cargo run --bin ppl-framework-host -- iam-workload \
  --state-dir "$PPL_M2_STATE" \
  --workload-id workload-director
```

The setup generates a distinct environment identity and local Ed25519 signing
key in protected local state. The health output prominently identifies the
`local-synthetic` profile. It fails readiness if declared as hosted, shared,
production-like, production or authorised for non-synthetic information. Run
`pnpm check:m2-runtime` for the complete temporary-environment conformance path;
it does not expose a reusable grant in terminal output.

## Repository layout

- `architecture/`: machine-readable component catalogue;
- `contracts/`: canonical schemas, examples, compatibility and conformance
  fixtures;
- `backend/`: Rust workspace and implemented component boundaries;
- `frontend/`: browser surfaces and shared TypeScript UI;
- `deploy/`: development containers, Compose and Kubernetes examples;
- `docs/architecture/logical/`: human-readable logical design; and
- `tools/`: repository architecture checks.

Machine-readable catalogue entries link design responsibilities to source
packages without asserting that every logical component is a service.

## Platform boundaries

No infrastructure product is selected yet for events, workflow, external or
managed identity, storage, retrieval, analytics or observability. The M1 and
M2 journals are single-host assurance bindings, not product selections.
Deployment examples contain only the framework host and static browser
surfaces. The same source is intended to build on macOS, Linux and Windows;
evidence from one host does not qualify another. Containers provide the common
hosted form.

cREXX remains the preferred future surface for appropriate inspectable rules,
transformations and scenario scripting. It is excluded from this initial build
until the RUL-01 runtime and integration contracts are designed.

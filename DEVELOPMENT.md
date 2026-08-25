# Development guide

## Current status

This repository contains a buildable framework with an M1 common-interaction
reference path in development. It proves common packaging, presentation
surfaces, architecture catalogues, strict common contracts, safe command
outcomes and local idempotency/restart behaviour while keeping unapproved
platform choices out of the implementation.

The M1 adapter is a local assurance profile with no network listener. It does
not authenticate its authority fixture and is not an event broker, API,
production audit store or distributed delivery mechanism. The repository does
not yet provide operational identity, synthetic sessions, workflow, business
persistence, retrieval, reporting, analytics or cREXX execution.

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

No infrastructure product is selected yet for events, workflow, identity,
storage, retrieval, analytics or observability. The M1 append journal is a
single-host assurance binding, not that product selection. Deployment examples
therefore contain only the framework host and static browser surfaces. The same
source is intended to build on macOS, Linux and Windows; evidence from one host
does not qualify another. Containers provide the common hosted form.

cREXX remains the preferred future surface for appropriate inspectable rules,
transformations and scenario scripting. It is excluded from this initial build
until the RUL-01 runtime and integration contracts are designed.

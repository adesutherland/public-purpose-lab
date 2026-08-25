# Development guide

## Current status

This repository contains a buildable framework skeleton, not an operational
platform. It proves common packaging, presentation surfaces, architecture
catalogues and deployment shapes while keeping unapproved technology choices
out of the implementation.

The skeleton does not yet provide identity, synthetic sessions, events,
workflow, persistence, retrieval, reporting, analytics or cREXX execution.

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
```

The `serve` command exists to qualify process and container lifecycle only; it
does not expose a network service.

## Repository layout

- `architecture/`: machine-readable component catalogue;
- `contracts/`: machine-readable contract catalogue and future schemas;
- `backend/`: Rust workspace and implemented component boundaries;
- `frontend/`: browser surfaces and shared TypeScript UI;
- `deploy/`: development containers, Compose and Kubernetes examples;
- `docs/architecture/logical/`: human-readable logical design; and
- `tools/`: repository architecture checks.

Machine-readable catalogue entries link design responsibilities to source
packages without asserting that every logical component is a service.

## Platform boundaries

No infrastructure product is selected yet for events, workflow, identity,
storage, retrieval, analytics or observability. Deployment examples therefore
contain only the framework host and static browser surfaces. The same source
builds on macOS, Linux and Windows; containers provide the common hosted form.

cREXX remains the preferred future surface for appropriate inspectable rules,
transformations and scenario scripting. It is excluded from this initial build
until the RUL-01 runtime and integration contracts are designed.

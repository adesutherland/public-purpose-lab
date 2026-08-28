# Development guide

## Current status

This repository contains a buildable framework with M1 common-interaction, M2
local-synthetic identity and M3.4 scenario, identity and presentation reference
paths in development. It proves common packaging, presentation surfaces,
architecture catalogues, strict contracts, safe command outcomes,
environment-scoped demonstration trust, durable application sessions,
backend-only synthetic sign-in and local idempotency/restart behaviour.

The M1/M2 adapters are local assurance profiles with no network listener. M2 can
establish bounded synthetic session state, but it is not a browser login,
external identity provider, managed issuer, production identity service or
distributed session store. M3.4 adds loopback or port-forwarded HTTP/SSE,
component-local sessions, a separate Identity Broker and a TLS/NKey NATS
JetStream development-assurance path. The managed OIDC/Cloud KMS adapter and
Kubernetes contract have bounded, synthetic-only hosted evidence; each future
activation still requires its protected configuration and admission checks.
The repository does not yet provide workflow, business persistence, retrieval,
reporting, analytics or cREXX execution.

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

## Exercise the M3.4 identity and presentation path

M3.4 requires the web bundles and `ppl-m3-runtime` binary, plus NATS tools for
the native path:

```sh
pnpm build:web
cargo build --bin ppl-m3-runtime
deploy/local/setup-m3-environment.sh .local/m3-environment native
deploy/local/start-m3-native.sh .local/m3-environment
tools/smoke-m3-native.sh
deploy/local/stop-m3-native.sh .local/m3-environment
```

Use a fresh environment-directory name when new synthetic trust is required;
setup refuses silent key rotation. For Minikube, OCI Compose and manual browser
exploration, follow the
[M3.4 operator guide](docs/guides/m3-4-operator-guide.md).

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

NATS JetStream is the accepted first M3 event binding and SQLite is the
accepted M3 single-instance component-state binding. Neither is a production,
multi-replica or long-term evidence-store qualification. No infrastructure
product is selected yet for workflow, retrieval, analytics or general
observability. Managed identity has a narrow M3.4 adapter, not a generally
qualified product. The M1 and M2 journals remain single-host
assurance bindings. The same source is intended to build on macOS, Linux and
Windows; evidence from one host does not qualify another. Containers provide
the common hosted form.

cREXX remains the preferred future surface for appropriate inspectable rules,
transformations and scenario scripting. It is excluded from this initial build
until the RUL-01 runtime and integration contracts are designed.

# M3.3 operator guide

Status: In-development, synthetic-only assurance guide

Last reviewed: 27 August 2026

## Purpose and limits

This guide starts, explores, smoke-tests and stops the M3.3 Scenario Director,
Presentation Surface and Workbench shell. It is suitable only for repository-
owned synthetic information. It does not establish production readiness,
managed identity, compliance, legal authority or a shared hosted service.

The local environments generate a distinct short-lived synthetic transport
root and separate Director and Presentation workload identities. The root is
valid only inside that environment. Its private key remains in the protected
environment directory and is never mounted into an application workload. The
interfaces and health output visibly identify the development-assurance trust
profile.

## What to explore

The three surfaces are:

| Surface         | Default local address               | Current capability                                                                                                                                                    |
| --------------- | ----------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Director        | `http://127.0.0.1:18081/`           | Create and progress a synthetic session, set or advance logical time, arm the bounded cue-delay fault, issue a semantic cue, inspect presentation progress and reset. |
| Presentation    | `http://127.0.0.1:18082/`           | Register an audience display, receive a semantic cue and return its bounded outcome.                                                                                  |
| Workbench shell | `http://127.0.0.1:18082/workbench/` | Register the second presentation role and demonstrate the common visual surface; asset, RAG, workflow and reporting capabilities are not implemented.                 |

Start with the Presentation Surface so it can register against the session
created in the Director. The Director and Presentation pages explain the small
assurance sequence. A successful presentation checkpoint means only that the
current cue was applied by the current registered surface. It is not evidence
of a business outcome.

The unauthenticated assurance endpoints are:

- `/health/live` for process health;
- `/health/ready` for local interactive dependency and trust-profile
  readiness; and
- `/health/contracts` for the admitted package or presentation manifest,
  contract list, source revision and image digest.

The physical HTTP/SSE adapter is specified in
[`contracts/http/m3-runtime.openapi.yaml`](../../contracts/http/m3-runtime.openapi.yaml).

## Native macOS or Linux

Prerequisites are the repository development toolchain plus `nats-server`,
`nsc`, OpenSSL, `curl` and `jq`. On macOS the NATS tools may be installed with
Homebrew; they are not mandated for hosted deployments.

From the repository root:

```sh
pnpm install --frozen-lockfile
pnpm build:web
cargo build --bin ppl-m3-runtime
deploy/local/setup-m3-environment.sh .local/m3-environment native
deploy/local/start-m3-native.sh .local/m3-environment
```

The setup command intentionally refuses to replace existing trust material.
Reuse the existing directory for restart testing or select a new directory for
a new environment.

Open the three addresses above, or run the complete HTTP/SSE path:

```sh
tools/smoke-m3-native.sh
```

Stop the processes while retaining their databases and broker state:

```sh
deploy/local/stop-m3-native.sh .local/m3-environment
```

Restart with `start-m3-native.sh` to inspect recovery. The environment is
ignored by Git and must never be copied into the repository.

## Local OCI Compose

Docker is not mandated. Use any Compose implementation that supports the
declared OCI build, secrets, read-only filesystems and named volumes.

Generate environment-scoped material using container paths, then build and
start:

```sh
PPL_M3_ENVIRONMENT_DIRECTORY="$(pwd)/.local/m3-compose"
export PPL_M3_ENVIRONMENT_DIRECTORY
deploy/local/setup-m3-environment.sh "$PPL_M3_ENVIRONMENT_DIRECTORY" portable
docker compose -f deploy/compose/m3-compose.yaml up --build --detach
tools/smoke-m3-native.sh
```

The portable setup keeps the environment directory at owner-only mode. Because
common Compose implementations expose file-backed secrets without applying
declared container ownership, only the NATS server key, workload seeds and
server configuration are made container-readable inside that protected
directory. The synthetic root private key remains owner-only and is never
mounted. Each workload receives only its own seed.

Substitute the equivalent command for another compatible Compose tool. Stop
the workloads while retaining the named volumes:

```sh
docker compose -f deploy/compose/m3-compose.yaml down
```

Deleting the named volumes or environment directory destroys local component
state or trust material and is outside the normal smoke-test cycle.

## Minikube

The supplied path uses Minikube with the `qemu2` driver and `containerd`, the
official pinned NATS Helm chart, separate Kubernetes Secrets and separate
`ReadWriteOnce` state claims. It creates only ClusterIP Services and requires
explicit port-forwarding.

One-time chart registration:

```sh
helm repo add nats https://nats-io.github.io/k8s/helm/charts/
helm repo update nats
```

Build the common image inside Minikube and deploy it:

```sh
deploy/local/start-m3-minikube.sh
```

For manual exploration, keep these two commands running in separate terminals:

```sh
kubectl -n public-purpose-lab port-forward service/m3-scenario-director 18081:8080
kubectl -n public-purpose-lab port-forward service/m3-presentation-gateway 18082:8080
```

Then open the three local addresses or run:

```sh
deploy/local/smoke-m3-minikube.sh
```

Stop the VM to stop cost-free local resource use while retaining the cluster:

```sh
deploy/local/stop-m3-minikube.sh
```

## Private hosted smoke

The provider-neutral public contract for M3.3 is deliberately narrow: use the
exact reviewed image digest and source revision; create no public endpoint;
prove liveness and contract/package self-test; prove that interactive readiness
and the development-session adapter fail closed without managed trust; capture
private inventory and cost evidence; then turn the environment off.

Provider credentials, project identifiers, billing evidence and activation
instructions belong in the private hosting repository. They must not be added
to this public guide. The hosted M3.3 profile is not an interactive demo; M3.4
must supply the separately reviewed managed trust and access bindings first.

## Safe failure and support

- A `503` with `managed-trust-binding-absent` is the expected hosted readiness
  result, not a liveness failure.
- A `401` from the development-session route in a hosted profile is required.
- A local readiness failure should be investigated through the bounded health
  responses and workload logs; do not print seed files, cookies or private
  keys.
- A failed reset must not be treated as clean. Retain the prior session and
  control outcomes for diagnosis.
- Never rename a local-synthetic profile to make it appear managed or suitable
  for real information.

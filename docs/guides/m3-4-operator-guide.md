# M3.4 operator guide

Status: In-development, synthetic-only assurance guide

Last reviewed: 28 August 2026

## Purpose and limits

This guide starts, explores, smoke-tests, restarts and stops the M3.4 Scenario
Director, Identity Broker, Presentation Surface and Workbench surface. It is
for repository-owned synthetic information only. It does not establish legal,
compliance, production, availability or non-synthetic-data authority.

Local profiles use visibly labelled test external identities and an
environment-generated local synthetic issuer. The `managed-hosted` profile
instead requires Google OIDC, a protected role map, GKE workload identity and a
retained Cloud KMS issuer; it refuses a local signer or development login.

## Surfaces and identities

| Surface           | Local address                                 | Required external role | Synthetic binding                                                 |
| ----------------- | --------------------------------------------- | ---------------------- | ----------------------------------------------------------------- |
| Director          | `http://localhost:18081/`                     | `presenter`            | Requests configured bindings but never receives a grant           |
| Audience display  | `http://presentation.localhost:18082/`        | `surface-operator`     | `synthetic-audience-user` on `audience-display`                   |
| Workbench surface | `http://workbench.localhost:18082/workbench/` | `surface-operator`     | `synthetic-reviewer` on `reviewer-workbench`                      |
| Identity health   | `http://127.0.0.1:18083/health/ready`         | no browser login       | Reports environment, trust profile, epoch and broker channel only |

The three local surfaces use distinct host origins while sharing the same
loopback environment. This preserves independent host-scoped application
sessions in one ordinary browser profile. It does not claim three independently
deployed application gateways.

Start the two surface sessions before requesting their synthetic sign-ins.
The Director sends only an identity request. The Identity Broker signs and
delivers the short-lived grant over the protected component channel, and the
target backend establishes it at most once. No browser receives the grant,
signature or synthetic session reference.

## Native macOS or Linux

Prerequisites are the repository toolchains plus `nats-server`, `nsc`, OpenSSL,
`curl` and `jq`.

```sh
pnpm install --frozen-lockfile
pnpm build:web
cargo build --bin ppl-m3-runtime
deploy/local/setup-m3-environment.sh .local/m3-environment native
deploy/local/start-m3-native.sh .local/m3-environment
tools/smoke-m3-native.sh
```

Setup refuses to replace existing trust material. Reuse the directory to test
ordinary restart or choose a new directory to create a new environment and
trust domain. An environment created before M3.4 has no Identity Broker
credential and is refused rather than upgraded in place; choose a new M3.4
directory so the earlier trust domain and evidence are not silently changed.

Stop all processes while retaining component, identity and broker state:

```sh
deploy/local/stop-m3-native.sh .local/m3-environment
```

Then run `start-m3-native.sh` again. Valid unexpired application state and
idempotency evidence survive; expired, revoked, stopped or superseded authority
does not. Deleting local state is not a recovery procedure and invalidates the
environment's evidence.

## Local OCI Compose

Docker is not mandated. Use a Compose implementation that supports the
declared OCI build, file-backed secrets, read-only filesystems and named
volumes.

```sh
PPL_M3_ENVIRONMENT_DIRECTORY="$(pwd)/.local/m3-compose"
export PPL_M3_ENVIRONMENT_DIRECTORY
deploy/local/setup-m3-environment.sh "$PPL_M3_ENVIRONMENT_DIRECTORY" portable
docker compose -f deploy/compose/m3-compose.yaml up --build --detach
PPL_DIRECTOR_ORIGIN=http://localhost:18081 \
  PPL_GATEWAY_ORIGIN=http://presentation.localhost:18082 \
  PPL_WORKBENCH_ORIGIN=http://workbench.localhost:18082 \
  tools/smoke-m3-native.sh
docker compose -f deploy/compose/m3-compose.yaml restart \
  nats identity-broker scenario-director presentation-gateway
PPL_DIRECTOR_ORIGIN=http://localhost:18081 \
  PPL_GATEWAY_ORIGIN=http://presentation.localhost:18082 \
  PPL_WORKBENCH_ORIGIN=http://workbench.localhost:18082 \
  tools/smoke-m3-native.sh
docker compose -f deploy/compose/m3-compose.yaml down
```

Normal `down` retains named state volumes. Removing volumes or the protected
environment directory is a deliberate destructive reset, not an ordinary stop.

## Minikube

The supplied path uses Minikube, containerd, the pinned official NATS chart,
separate Secrets and separate `ReadWriteOnce` claims.

```sh
helm repo add nats https://nats-io.github.io/k8s/helm/charts/
helm repo update nats
deploy/local/start-m3-minikube.sh
```

For exploration, keep these port-forwards open:

```sh
kubectl -n public-purpose-lab port-forward service/m3-identity-broker 18083:8080
kubectl -n public-purpose-lab port-forward service/m3-scenario-director 18081:8080
kubectl -n public-purpose-lab port-forward service/m3-presentation-gateway 18082:8080
```

Then run `deploy/local/smoke-m3-minikube.sh`. Stop the local VM with
`deploy/local/stop-m3-minikube.sh`; no Ingress or LoadBalancer is created.

## Managed-hosted deployment contract

The public portable binding is
[`deploy/kubernetes/m3/overlays/managed-hosted`](../../deploy/kubernetes/m3/overlays/managed-hosted/README.md).
It deliberately references protected configuration that only the private
infrastructure repository may supply. A structural render is not authorisation
to deploy it:

```sh
kubectl kustomize deploy/kubernetes/m3/overlays/managed-hosted
```

Before a shared activation, the private operator procedure must prove the
exact image digest and source revision, HTTPS origins/callbacks, named OIDC
role mappings, environment trust record, exact KMS issuer version, dedicated
Kubernetes workload identity, least-privilege NATS identities, automatic
expiry and the approved maximum duration. `off` is conclusive only after the
cluster, endpoint, forwarding rule, activation disk and workloads are absent.

Never place OIDC secrets, subject identifiers, NKey seeds, KMS project/key
configuration, raw cloud evidence or provider credentials in this public
repository.

The first managed-hosted cycle is recorded in the
[M3.4 managed-hosted evidence](../architecture/evidence/m3-4-managed-hosted-identity-and-resilience.md).
Its temporary endpoints and activation infrastructure are off. The record is a
repeatable assurance baseline, not authority to leave a shared environment
running or to introduce real information.

## Safe failure

- `identity-binding-unavailable` means the OIDC or public synthetic-trust
  dependency is absent or inconsistent.
- `event-broker-unavailable` means interaction must stop; health is not
  scenario readiness.
- `external-identity-not-authorised` does not enumerate the protected role map.
- A successful synthetic sign-in proves only an application binding for a
  configured actor, purpose and surface.
- A successful presentation checkpoint proves presentation progress, not
  human attention, business completion or compliance.
- Broker volume loss, damaged SQLite state, missing trust material or uncertain
  teardown remains a visible recovery item and must not be bypassed.

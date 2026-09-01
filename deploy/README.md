# Deployment baselines

For the current synthetic source-intake increment, see the
[Gate C governed source-intake guide](../docs/guides/gate-c-source-intake.md).

The deployment baselines prove that the current Rust components and browser
surfaces can be packaged in containers and arranged in Kubernetes-compatible
shapes. They are not production topologies or security qualifications.

The original M1/M2 skeleton contains only:

- an in-development Rust framework host with local M1 common-interaction and M2
  local-synthetic identity reference paths and one declared state mount; and
- one static web container serving the Workbench, Director and Presentation
  builds under separate paths.

It deliberately has no network listener, event broker, workflow engine,
database, vector store, external identity provider, ingress, managed
certificate authority or observability stack. Environment setup generates a
local-synthetic Ed25519 root and demo actor configuration inside the state
mount. This is local secret material, not a Kubernetes Secret or a managed
issuer. The append journals prevent duplicate application for the single-host
assurance adapters; they are not authoritative audit stores or distributed
persistence choices.

IAM-01 and INT-01 use separate protected subdirectories within the shared
state mount. This lets the non-root processes protect their own state without
trying to take ownership of a container runtime or Kubernetes-managed volume
root.

Build and run locally with an OCI-compatible Compose implementation:

```sh
docker compose -f deploy/compose/compose.yaml up --build
```

The surfaces are then available under `http://localhost:8080/`. Render the
Kubernetes base with:

```sh
kubectl kustomize deploy/kubernetes/base
```

An init step idempotently configures IAM before the framework host becomes
ready, and health checks require the identity path. The Compose named volume
persists the local trust material and journals across container restart. The
Kubernetes `emptyDir` preserves them across process/container restart in one
Pod but not Pod replacement: replacement deliberately creates a new
environment trust domain and invalidates former grants and sessions. The image
names are placeholders for locally built or future registry images.

These manifests are strictly `local-synthetic` examples even when rendered for
Kubernetes. They must not be relabelled for hosted, shared, production-like,
production or non-synthetic-data use; the runtime fails that declaration.
Those profiles require a separately approved managed trust binding, protected
persistence and recovery design.

## M3.4 runtime deployment

M3.4 extends the synthetic-only walking-skeleton composition:

- `deploy/containers/m3-runtime.Containerfile` builds the Director, Identity
  Broker, Presentation Gateway and three browser bundles into one immutable
  image;
- `deploy/compose/m3-compose.yaml` runs that image in three M3 modes and nine
  configurable Gate A component-host modes with separate SQLite volumes and
  workload seeds, plus TLS/NKey NATS JetStream; a
  network-isolated one-shot initializer gives those empty volumes to the
  image's non-root runtime identity before any application starts;
- `deploy/kubernetes/m3/base/` supplies ingress-free Kustomize application
  resources with separate `ReadWriteOnce` claims;
- `deploy/kubernetes/m3/overlays/managed-hosted/` defines the portable
  protected-configuration and GKE workload-identity contract without public
  account values or secrets;
- `deploy/kubernetes/m3/nats-values.yaml` pins the official NATS chart and
  file-store limits; and
- `deploy/local/` contains environment setup, Minikube and native lifecycle
  helpers.

Local setup generates environment-scoped transport and synthetic trust
material. Private keys stay in the ignored environment/state directories and
are not mounted into the Director or Presentation workload. Each workload
receives only its own NKey seed; target applications receive only public
synthetic trust. This trust is not portable to another environment and must not
be used for real information.

The full start, explore, smoke and stop procedures are in the
[M3.4 operator guide](../docs/guides/m3-4-operator-guide.md). The managed-hosted
overlay is not directly deployable: the private infrastructure repository must
supply exact HTTPS, OIDC, managed-trust, NATS, immutable-image, expiry and
teardown bindings. A successful public render is structural evidence only.

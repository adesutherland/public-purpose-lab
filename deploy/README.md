# Deployment skeleton

The deployment baseline proves that the current Rust host and browser surfaces
can be packaged in containers and arranged in a Kubernetes-compatible shape.
It is not a production topology or security qualification.

The current deployment contains only:

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

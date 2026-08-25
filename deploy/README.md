# Deployment skeleton

The deployment baseline proves that the current Rust host and browser surfaces
can be packaged in containers and arranged in a Kubernetes-compatible shape.
It is not a production topology or security qualification.

The initial deployment contains only:

- an in-development Rust framework host with the local M1 command/outcome
  reference path and one declared state mount; and
- one static web container serving the Workbench, Director and Presentation
  builds under separate paths.

It deliberately has no network listener, event broker, workflow engine,
database, vector store, identity provider, ingress, certificate authority,
secrets or observability stack. The append journal prevents duplicate
application for the single-host assurance adapter; it is not an authoritative
audit store or distributed persistence choice.

Build and run locally with an OCI-compatible Compose implementation:

```sh
docker compose -f deploy/compose/compose.yaml up --build
```

The surfaces are then available under `http://localhost:8080/`. Render the
Kubernetes base with:

```sh
kubectl kustomize deploy/kubernetes/base
```

The Compose named volume persists the reference journal across container
restart. The Kubernetes `emptyDir` preserves it across process/container
restart in one Pod but not Pod replacement; that limitation is deliberate and
visible. The image names are placeholders for locally built or future registry
images. Environment overlays, ingress, qualified persistence and secret
material follow only when their contracts are approved. No synthetic root
certificate is created by this baseline.

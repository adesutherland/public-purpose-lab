# Deployment skeleton

The deployment skeleton proves that the current Rust host and browser surfaces
can be packaged in containers and arranged in a Kubernetes-compatible shape.
It is not a production topology or security qualification.

The initial deployment contains only:

- a lifecycle-only Rust framework host; and
- one static web container serving the Workbench, Director and Presentation
  builds under separate paths.

It deliberately has no event broker, workflow engine, database, vector store,
identity provider, ingress, certificate authority, secrets or observability
stack. Those additions require scenario evidence and architecture decisions.

Build and run locally with an OCI-compatible Compose implementation:

```sh
docker compose -f deploy/compose/compose.yaml up --build
```

The surfaces are then available under `http://localhost:8080/`. Render the
Kubernetes base with:

```sh
kubectl kustomize deploy/kubernetes/base
```

The Kubernetes image names are placeholders for locally built or future
registry images. Environment overlays, ingress, persistence and secret
material will follow only when their contracts are approved. In particular,
no synthetic root certificate is created by this skeleton.

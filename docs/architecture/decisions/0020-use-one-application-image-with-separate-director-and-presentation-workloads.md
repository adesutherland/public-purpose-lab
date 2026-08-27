# ADR-0020: Use one application image with separate Director and presentation workloads

Status: Accepted

Date: 2026-08-27

## Context

M3.3 needs a Director, Presentation Gateway, browser applications, component
event transport and durable control state. The implementation should remain
small, portable and cheap while preserving the logical ownership and identity
boundaries already accepted.

Running `CTL-01` and `CTL-02` in one process would be mechanically simple, but
both NATS identities and databases would then sit in one compromise and restart
boundary. It would also make the event adapter largely an internal loop rather
than evidence of an explicit component contract.

Building separate products for each logical component would duplicate build,
container and dependency work before either has independent scaling or release
needs.

The browser channel must be backend-mediated and same-origin. Adding a general
edge proxy solely to combine static assets and APIs would introduce another
component before it adds scenario value.

## Decision

Build one immutable M3.3 application image containing the Rust runtime, database
migrations, contract schemas/fixtures, the first scenario package and compiled
Director, Presentation and Workbench frontend assets.

Run that image in two modes and processes:

- `scenario-director`, owning `CTL-01`, its SQLite database, Director API/UI and
  Director NATS identity/permissions; and
- `presentation-gateway`, owning `CTL-02`, its SQLite database, surface
  API/SSE/UI and Presentation Gateway NATS identity/permissions.

The modes share code and image layers, not databases, credentials or mutable
state. Mode configuration is explicit and closed; an unknown or conflicting
mode fails startup. Each process runs as a non-root user with a read-only root
filesystem, one writable state mount, no default Kubernetes API token and only
its required network and NATS access.

NATS JetStream runs as a third service. Local-container and Minikube profiles
use one server with file storage, TLS and separate environment-generated NKey
identities/subject permissions for the two application workloads. Browser code
has no NATS network path or credential.

Each backend serves its owning compiled frontend assets and versioned API from
one origin. Development Vite servers may proxy to loopback backends. There is no
new release reverse proxy in M3.3.

Use one replica for each application mode. Local containers use Compose and
named volumes. Minikube uses Kustomize for application resources, the accepted
pinned official NATS Helm chart and separate `ReadWriteOnce` claims. Services
are `ClusterIP`; operators use explicit port-forwarding for local interaction.

The private Google Cloud M3.3 overlay uses the exact application image digest
but creates no Ingress, public IP or LoadBalancer. Until M3.4 managed trust is
present, it runs only ingress-free liveness, package/schema self-test and
expected fail-closed readiness checks. It does not enable a local-synthetic
root, development presenter adapter, interactive browser path or NATS
application traffic in the hosted environment.

The Workbench remains a shell and second presentation surface only. M3.3 adds
no asset, retrieval, report or business authority to it.

## Alternatives considered

- **One process for `CTL-01` and `CTL-02`:** fewer processes but combines state,
  NATS credentials and restart/failure authority and weakens the intended
  component evidence.
- **Separate codebases and images:** stronger release separation, but duplicates
  packaging and dependency work without current ownership or scaling evidence.
- **Static web container plus reverse proxy:** preserves a separate asset image
  but adds routing/configuration/security work solely to achieve same-origin
  API access.
- **Direct browser-to-NATS:** rejected by ADR-0013 and ADR-0015 because it
  exposes transport credentials and bypasses application-session enforcement.
- **Public Minikube/GKE ingress in M3.3:** rejected because interactive hosted
  trust and presenter identity are M3.4 gates.
- **Use Kubernetes Events instead of NATS:** rejected by ADR-0013; they are
  operational diagnostics rather than application contracts.

## Consequences

- One build produces the same application and package artifact for both
  backend roles and all profiles.
- Separate processes make state, permissions, restart and event-flow evidence
  meaningful without creating separate products.
- The image is larger because it carries both modes and frontend assets; image
  size and vulnerability scanning become acceptance evidence.
- A compromise of the shared code dependency can still affect both roles, even
  though runtime credentials and state are separated.
- Serving static assets from Rust avoids a release proxy but adds bounded
  static-file and content-security-policy responsibilities to each backend.
- Hosted M3.3 proves packaging and fail-closed profile behaviour, not functional
  hosted presentation parity. That functional gate remains M3.4.
- A later split into separate images is possible if release, scaling, ownership
  or security evidence justifies it.

## Validation and review

Evidence must demonstrate:

- identical image and package digests for both modes and across local,
  Minikube and private hosted runs;
- mode-confusion refusal and absence of the other mode's database, NATS seed
  and publish/subscribe permission;
- independent process/pod restart and reconciliation;
- no browser NATS path, credential, private subject or backend route disclosure;
- same-origin SSE/POST, origin/CSRF protection and bounded reconnect in local
  profiles;
- non-root/read-only filesystem, explicit writable mounts, resource bounds and
  no default Kubernetes API token;
- `ClusterIP`/port-forward-only Minikube access; and
- no Google Cloud application endpoint, development adapter, synthetic root or
  residual workload/volume after the hosted M3.3 smoke is turned off.

Review the composition before multiple replicas, independent releases,
separate teams, a dedicated backend-for-frontend, shared hosted access or
materially different scaling/security needs arise.

# ADR-0016: Bind the first hosted preview to ephemeral GKE Autopilot

Status: Accepted
Date: 2026-08-27

## Context

ADR-0012 selects a cost-controlled Google Cloud preview during M3 but leaves
the services, managed trust, operator pipeline and infrastructure definition
open. The application architecture is Kubernetes-oriented and already has a
Kustomize base. The preview should test that portability without making a
continuously running cluster the normal development environment.

The founder-funded operating target is approximately 30 units in the billing
account's currency per month on average. Credits can reduce the bill but cannot
hide gross resource cost. Authorised people need a simple attributable `on`
and `off` operation, with independent automatic expiry if the initiating
session disappears.

A shared hosted preview must use `managed` trust. It must never import or
promote a local-synthetic root. The environment's managed trust identity and
required audit/recovery state need to survive an ordinary `off`/`on` cycle,
while application workloads and public endpoints should not.

## Decision

Use an **ephemeral GKE Autopilot cluster** for the first hosted application
runtime. `off` deletes the cluster and all activation-scoped runtime resources;
`on` recreates them from versioned definitions.

Use **OpenTofu** for Google Cloud infrastructure and the repository's existing
**Kustomize** model for Kubernetes resources. Account-specific state,
identifiers, budget settings and operator mappings remain in protected
infrastructure configuration. Portable requirements, overlays and conformance
checks remain in this public repository.

### Two infrastructure states

Keep two separately owned and locked OpenTofu states:

1. **Retained foundation** — project/service enablement, Workload Identity
   Federation configuration, dedicated lifecycle service accounts, versioned
   GCS state/evidence buckets, Artifact Registry, Cloud KMS trust keys and
   public certificates, budgets/alerts, Cloud Tasks expiry queue, teardown
   trigger and safe audit configuration.
2. **Ephemeral activation** — GKE Autopilot cluster, activation-specific
   networking and ingress, namespaces, workloads, services, temporary data and
   any runtime endpoint.

The GCS state bucket uses object versioning, locking and restricted access.
State and plan material is treated as sensitive and never published. No secret,
private key or reusable credential is stored in OpenTofu configuration or
outputs.

### `on`, expiry and `off`

An `on` request:

- names the operator, immutable application artifact digest, infrastructure
  revision, purpose and maximum expiry;
- defaults to a four-hour lifetime and cannot exceed eight hours in the first
  profile without a separately approved extension;
- uses a protected GitHub environment/manual workflow and Google Cloud
  Workload Identity Federation, restricted to the approved repository,
  revision/ref and deployment role;
- applies the activation state, then the GKE Kustomize overlay;
- records endpoint, readiness, gross-cost forecast and expiry evidence; and
- refuses shared synthetic sign-in until managed trust, presenter and workload
  readiness pass.

The same workflow performs a requested `off`. At activation it also creates a
one-time **Cloud Task** scheduled for the recorded expiry. The task uses a
dedicated identity to invoke a pinned, least-privilege **Cloud Build teardown
trigger**. The cloud-side build uses a dedicated user-specified service account
and the same immutable OpenTofu activation definition. This is an independent
safety path, not a second infrastructure authority. Successful teardown
concludes the task; bounded task retries handle transient failure, while an
unresolved result remains an alert and recovery item.

`off` stops ingress and application work, exports only approved safe evidence,
destroys the activation state and verifies that no cluster, external runtime
endpoint, forwarding rule, activation disk or application workload remains.
Repeated `on` and `off` requests reconcile the same operation. An interrupted
or uncertain teardown remains visibly not-off until inventory proves the
outcome.

No `warm-off` profile is selected. If startup measurements later justify one,
it requires a separate costed decision.

### Kubernetes and workload identity

Autopilot is selected because it preserves Kubernetes manifests while Google
manages nodes and can scale a cluster with no running workloads to zero nodes.
The stronger `off` state still deletes the cluster to remove the management
surface and avoid depending on a free-tier management-fee credit.

GKE Workload Identity Federation is mandatory. Each component uses a dedicated
Kubernetes service account and narrowly scoped Google Cloud IAM grants. No
application Pod uses a node identity, exported Google service-account key or
`hostNetwork`. Namespace-only IAM grants are avoided where a specific service
account can be named, and cross-cluster/project identity sameness is tested.

### Managed environment trust

The retained foundation creates a new `managed` trust domain for the hosted
environment. It uses Cloud KMS software-protected `EC_SIGN_ED25519` asymmetric
keys so the current M2 Ed25519 grant profile can remain compatible while
private key material is non-exportable.

Google documents Ed25519 as supported but recommends P-256 for general elliptic-
curve signing. This bounded choice preserves the accepted `ppl-i004-ed25519-v1`
contract and permits direct local/hosted conformance comparison. Algorithm
agility and the provider recommendation are reviewed before any production
qualification; changing algorithm creates a new signature profile rather than
silently changing the current one.

The bootstrap path creates:

- an environment root key and self-signed public root certificate generated as
  part of that environment's setup;
- a separate environment issuer key and certificate signed by the root;
- environment/trust-domain identifiers, key versions, validity, rotation,
  revocation, custody and recovery evidence; and
- distinct least-privilege identities: bootstrap may use the root only for
  bounded issuer administration, while the runtime grant signer may use only
  the current issuer for the declared `I-004` purpose.

The root key is disabled for routine signing after issuer bootstrap and is not
mounted into GKE. The runtime signer calls Cloud KMS through Workload Identity
Federation and cannot export either private key. Public certificates and key
versions are available to validators. Rotation does not silently change the
environment or trust-domain identity.

The managed root and issuer normally survive `off`; otherwise every activation
would create a new trust domain and invalidate retained security evidence. A
deliberate final environment destruction terminates the trust domain, records
the outcome and follows the key-destruction/recovery policy. It is not an
ordinary cost-saving `off` operation.

Operations and evidence views visibly report `managed`, current trust epoch,
issuer readiness and any failure. A local-synthetic key, missing KMS key,
wrong project/key version or unproven recovery state fails readiness closed.

### Data, persistence and retained cost

M3 hosted application data remains synthetic and resettable. The first preview
does not select a managed database or persistent application volume. Runtime
control state needed only for the disposable smoke is activation-scoped; safe
milestone evidence is exported deliberately before `off`.

The retained foundation can incur cost for Artifact Registry storage, GCS,
Cloud KMS active key versions/operations, task/build invocations and audit
storage. Each activation report separates:

- gross cost and usage before credits;
- credits, including any GKE free-tier credit;
- net billed cost;
- activation duration and forecast versus actual;
- retained-foundation cost while `off`; and
- cleanup exceptions and still-chargeable resources.

Budget alerts and provider controls are defence in depth, never the off
mechanism. The static public website remains separately hosted and cannot
activate or keep the preview running.

Before each activation, the workflow produces a current-price forecast for the
declared lifetime and refuses an activation that exceeds the protected monthly
operating limit unless a named founder records a bounded exception. A credit is
shown separately and is not assumed to continue.

## Alternatives considered

- **Long-lived GKE Autopilot cluster with zero workloads:** operationally
  quicker, and nodes can reach zero, but retains the cluster surface and gross
  management fee. It may be reconsidered only as an evidenced `warm-off`.
- **GKE Standard:** offers more node control but adds node-pool sizing,
  lifecycle and idle cost before a scenario needs it.
- **Cloud Run for the whole application:** attractive scale-to-zero economics,
  but would not test the agreed Kubernetes-hosted component model. Cloud Run
  remains suitable for narrowly justified independent jobs later.
- **Minikube on a Compute Engine VM:** resembles local operation but adds VM
  administration and does not exercise a managed Kubernetes profile.
- **Terraform instead of OpenTofu:** technically compatible, but OpenTofu gives
  the open-source-intended project an open implementation baseline while using
  the same provider model.
- **Certificate Authority Service:** stronger managed PKI facilities but adds
  cost and capability beyond the bounded M3 grant trust path. Reconsider for a
  wider certificate estate or production qualification.
- **Cloud KMS root signing every grant:** rejected because it makes the root a
  routine online signing identity; a separate environment issuer limits use.

## Consequences

- Cluster startup time becomes part of activation and must be presented
  honestly. The normal development loop stays local.
- The GKE management fee accrues only while the cluster exists, while retained
  foundation resources have small but non-zero continuing cost.
- Environment trust remains stable across ordinary cost-saving off/on cycles,
  while application state is deliberately disposable in the first profile.
- OpenTofu state, lifecycle IAM, expiry builds and KMS recovery become protected
  operational assets requiring backup, audit and least-privilege tests.
- Cloud-specific trust and pipeline adapters remain outside portable component
  contracts and Kubernetes application manifests.
- This decision does not select a production database, high-availability NATS,
  public access policy or formal backup/restore qualification.

## Validation and review

M3.2 performs one infrastructure-only create/destroy spike after publication of
this decision. It must record duration, gross forecast/actual cost, created
resource inventory, automatic-expiry setup, destroyed inventory and residual
foundation resources. It creates no shared application or synthetic sign-in
claim.

Before M3.3 private application smoke, evidence must show reproducible state,
artifact-digest deployment, dedicated workload identities, local/hosted
contract parity and conclusive off. Before M3.4 shared use, evidence must also
show Google presenter authentication, managed trust bootstrap, KMS signing,
cross-environment refusal, protected state/recovery and authorised activation.

Review the choice after three representative activations or sooner if startup
time, residual cost, Autopilot constraints, teardown uncertainty or the managed
issuer implementation defeats the M3 scope or average cost target.

## Reference material

- [GKE Autopilot overview](https://docs.cloud.google.com/kubernetes-engine/docs/concepts/autopilot-overview)
- [GKE pricing](https://cloud.google.com/kubernetes-engine/pricing)
- [Workload Identity Federation for GKE](https://docs.cloud.google.com/kubernetes-engine/docs/concepts/workload-identity)
- [Cloud KMS signing algorithms](https://docs.cloud.google.com/kms/docs/algorithms)
- [Cloud KMS asymmetric signing API](https://docs.cloud.google.com/kms/docs/reference/rest/v1/projects.locations.keyRings.cryptoKeys.cryptoKeyVersions/asymmetricSign)
- [OpenTofu GCS backend](https://opentofu.org/docs/language/settings/backends/gcs/)
- [Cloud Tasks scheduled HTTP tasks](https://docs.cloud.google.com/tasks/docs/creating-http-target-tasks)
- [Cloud Build user-specified service accounts](https://docs.cloud.google.com/build/docs/securing-builds/configure-user-specified-service-accounts)

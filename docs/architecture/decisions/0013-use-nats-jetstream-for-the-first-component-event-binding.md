# ADR-0013: Use NATS JetStream for the first component event binding

Status: Accepted
Date: 2026-08-27

## Context

M3 needs one replaceable component-event binding that operates on a developer
machine, Minikube and the first Kubernetes-hosted preview. It must carry
versioned commands, facts and outcomes with correlation, expiry and
redelivery, while leaving authority and state ownership in receiving
components.

Kubernetes Events are operational diagnostics, not an application message bus.
Direct browser-to-broker control would expose infrastructure and bypass
application-session and `CTL-02` enforcement. A large streaming platform would
add operational and cost weight before the demonstrator needs its scale.

The first implementation needs durable at-least-once delivery and explicit
acknowledgement. It does not need global ordering, unbounded history or a claim
of end-to-end exactly-once execution.

## Decision

Use NATS with JetStream as the first `INT-01` component event and durable
command-delivery binding.

The binding has these rules:

- Rust services use the maintained NATS Rust client through an `INT-01`
  adapter; domain code depends on common contracts, not NATS APIs or subjects.
- JetStream carries component commands, facts and outcomes that require
  restart-safe delivery. Core NATS fire-and-forget delivery is not used where
  loss would create ambiguous scenario or business state.
- Consumers use explicit acknowledgement. A receiver acknowledges only after
  its durable semantic decision or safe inbox record exists.
- Delivery is treated as **at least once**. `C-001` message identity, immutable
  content digest, scoped idempotency and receiver state provide end-to-end
  duplicate protection. Broker acknowledgement or de-duplication is useful
  evidence but never substitutes for receiver idempotency.
- Contract expiry is checked by publishers, adapters and receivers against
  protected operational time. Stream retention bounds storage but does not
  extend message validity.
- Ordering is declared only for a component-owned aggregate or the current
  Demonstration Session/surface-slot revision. No global subject or stream
  ordering is assumed.
- Terminal refusal, poison-message and uncertain-delivery handling produces
  bounded contract outcomes and support evidence. Payloads are not copied into
  an unrestricted dead-letter archive.
- Browsers never connect to NATS. An application backend or backend-for-
  frontend translates an authorised component message into the presentation
  channel selected by ADR-0015.

The first topology is intentionally small:

- one NATS/JetStream server with file storage for local-container and Minikube
  development;
- one NATS/JetStream server for the synthetic M3 hosted demonstrator while the
  environment is `on`; and
- no claim of broker high availability. Loss or uncertainty is visible and
  exercises recovery behaviour.

Before a shared hosted demonstration, storage, resource requests, maximum
payload, retention, stream replicas and recovery evidence are fixed from
measurements. A three-server JetStream cluster is added only when the assurance
scenario or availability target justifies its cost and operational surface.

Kubernetes profiles use a pinned release of the official `nats-io/k8s` NATS
Helm chart with reviewed values and rendered-manifest checks. Application
resources continue to use the repository's Kustomize base and overlays. The
chart is a replaceable packaging dependency; its defaults are not accepted as
the Lab's security, persistence or resource policy.

The first profile uses NATS centralised authentication with one environment-
generated NKey identity per workload. The server configuration retains only
each public NKey and its allow-listed publish/subscribe subjects; the matching
private seed is mounted only into that workload through the environment's
protected secret mechanism. Seeds are never committed, shared between
workloads or available to frontend code. TLS protects connections and server
identity. Operator/JWT mode is deferred until user or tenancy scale justifies
its additional authority and lifecycle.

NATS subjects are deployment configuration, not contract content, and must not
appear in scenario packages, presentation cues or public evidence. Credentials
are generated and rotated per environment.

The initial subject convention may include environment, contract family and
message class to support least privilege, but it is private binding metadata.
It cannot become the semantic identifier or authority source for a message.

## Alternatives considered

- **HTTP-only service calls:** simpler for synchronous operations but does not
  provide the durable decoupled fact/outcome and reconnect evidence required by
  M3 without building a comparable queue.
- **PostgreSQL outbox and inbox as the bus:** valuable for component atomicity,
  and may still be used inside an owner, but makes one database an integration
  transport and ownership risk too early.
- **RabbitMQ:** capable and mature, but its first topology and protocol surface
  add more policy than this small demonstrator currently needs.
- **Apache Kafka:** strong event-log capabilities but disproportionate for the
  initial scale, local footprint and cost target.
- **Redis Streams:** operationally small, but combines cache/data and durable
  transport concerns and offers a less direct fit for the intended subject-
  permission model.
- **Kubernetes Events:** rejected for application contracts because their
  purpose, retention and authority do not meet the scenario requirements.

## Consequences

- Local and Kubernetes profiles gain one small, well-supported broker binding.
- Every receiver still implements idempotency, expiry, authority and uncertain-
  outcome reconciliation; JetStream does not make application effects exactly
  once.
- Single-server M3 profiles can lose availability and require recovery. That is
  an explicit demonstrator limitation, not hidden resilience.
- NATS identity, TLS, subject permissions, retention and resource limits become
  security configuration that requires negative tests and safe evidence.
- Browser delivery remains a separate, narrower channel and does not inherit
  broker credentials or replay semantics.
- The adapter boundary permits replacement if measured complexity, licensing,
  performance, availability or hosted cost is unsuitable.

## Validation and review

Before acceptance as an implemented binding, evidence must demonstrate:

- local-container and Minikube start, restart and clean rebuild;
- durable redelivery after publisher, broker and consumer interruption;
- identical duplicate handling and changed-content conflict refusal;
- expiry during queue delay and no scenario-logical-time influence;
- receiver reconciliation after lost acknowledgement;
- publish and subscribe denial outside each workload's allow-list;
- bounded retention, payload, consumer backlog and failure evidence;
- no direct browser connection or broker credential in frontend artifacts;
- disclosure scanning of messages, advisories, logs and support views; and
- the M3 assurance path using the same contracts through the adapter.

Review the single-server limitation before M3.4, and revisit the product choice
if a scenario requires materially different routing, replay, tenancy,
availability or operational economics.

## Reference material

- [NATS JetStream concepts](https://docs.nats.io/concepts/jetstream)
- [NATS security and authentication](https://docs.nats.io/learn/security/authentication-basics)
- [Official NATS Kubernetes Helm chart](https://github.com/nats-io/k8s/tree/main/helm/charts/nats)

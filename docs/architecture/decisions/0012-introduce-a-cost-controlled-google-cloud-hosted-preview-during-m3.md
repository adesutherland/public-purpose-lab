# ADR-0012: Introduce a cost-controlled Google Cloud hosted preview during M3

Status: Accepted
Date: 2026-08-27

## Context

The initial roadmap requires one logical product to operate locally and in a
Kubernetes-compatible hosted profile. Waiting until M6 to exercise Google Cloud
would defer portability, identity, recovery, networking and cost evidence until
late in the roadmap. Making cloud hosting the primary development environment
now would add cost and provider-specific complexity before the M3 control and
presentation bindings are settled.

The hosted demonstrator must normally be inactive, be activated only by named
authorised people and return automatically to a low-cost state. The initial
founder-funded planning target is approximately 30 units in the billing
account's currency per month on average. Credits may help meet that target but
must not conceal the gross resource cost or become an architectural dependency.

The public website and the demonstrator have different availability, security
and cost characteristics. An always-available static website must not keep the
demonstrator running.

## Decision

Use Google Cloud as the first hosted-preview environment, introduced in stages
rather than as the primary development platform:

1. **M3.2 — hosted lifecycle spike:** define the hosted profile, operator
   authority, trust, cost, activation and teardown contracts and perform one
   disposable infrastructure create/destroy exercise.
2. **M3.3 — private walking-skeleton smoke:** after the local runtime walking
   skeleton passes, deploy the same versioned artifacts privately, run bounded
   health and contract checks and deactivate the environment.
3. **M3.4 — authorised hosted demonstration:** add managed trust, presenter and
   workload authentication, protected state and an attributable operator
   activation workflow before permitting a shared demonstration.
4. **M3.5 — scheduled evidence:** activate for a bounded demonstration, capture
   operational and cost evidence and automatically deactivate.
5. **M6 — supported qualification:** retain formal local/hosted support,
   backup, restore, observability and operational qualification in M6.

The initial lifecycle has two authoritative states:

- **`off`** is the default. Application workloads and publicly reachable
  runtime endpoints are absent. Only deliberately retained artifacts,
  configuration, protected recovery material and approved evidence may remain
  and incur residual cost.
- **`on`** is an attributable, time-bounded activation of one declared
  environment version. Every activation records an automatic expiry and a
  conclusive deactivation outcome.

An optional **`warm-off`** state may be added only when measurements show that
faster activation justifies its continuing cost. It is never described as zero
cost.

Activation and deactivation require named operator authority and a
least-privilege deployment workload using short-lived federated credentials.
No personal access token, long-lived service-account key or browser credential
is stored as a shared deployment secret. Repeated activation or deactivation is
idempotent, and uncertain teardown remains visible until reconciled.

Every shared or hosted-for-others preview uses the `managed` trust profile
accepted in ADR-0007: an accountable managed root or upstream trust service and
an environment-scoped issuing identity, with defined custody, rotation,
revocation, recovery and audit. A local-synthetic root cannot be copied,
relabeled or promoted into the hosted preview. Before the managed binding is
ready, a private infrastructure smoke test may run only with identity readiness
failed closed and no shared synthetic sign-in.

The operating target is an average net hosted-preview cost of approximately 30
units in the billing account's currency per month. Cost evidence records:

- gross resource usage before credits;
- applied credits separately;
- net billed cost;
- cost while `on`, residual cost while `off` and activation duration;
- forecast and actual cost by material service; and
- cleanup exceptions and resources that continue charging.

Automatic expiry, bounded replicas/resources, teardown checks, budgets, alerts
and provider spend controls are defence in depth. A credit, budget alert or
provider cap is not the primary off mechanism. Exact budget thresholds and
billing-account details remain private operational configuration rather than
public repository content.

This decision selects Google Cloud for the first preview and the lifecycle
principles only. It does not yet select GKE mode, Cloud Run responsibilities,
database, storage, load balancer, event transport, infrastructure-as-code tool,
managed issuer or deployment pipeline product. Those bindings require M3.2 or
later ADRs and measured evidence.

## Consequences

- Cloud portability and cost behaviour are tested before formal M6
  qualification without making cloud operation the normal development loop.
- The local environment remains the fastest and cheapest implementation and
  contract-test path.
- Environment creation and teardown must be reproducible; hand-configured
  resources cannot be the authoritative deployment.
- Startup latency is accepted in the default `off` posture. Warm capacity must
  earn its cost through evidence.
- The demonstrator and static public website remain independently deployable
  and cannot keep one another active.
- A shared hosted demonstration cannot precede the managed trust and
  authentication gate even when all scenario data is synthetic.
- The approximate monthly target guides design and monitoring but is not a
  promise that every month or individual activation will have the same cost.
- Exact account, credit-balance, budget and identity configuration belongs in
  protected infrastructure records; the public repository retains portable
  requirements and safe evidence only.

## Validation and review

Evidence must demonstrate:

- create, activate, expire, deactivate and recreate from versioned definitions;
- only authorised operators can change lifecycle state;
- repeated and interrupted lifecycle operations reconcile safely;
- `off` leaves no unintended runtime endpoint, workload or material charging
  resource;
- retained state and evidence are explicit and recoverable without cloning a
  trust domain;
- a local-synthetic root cannot make hosted identity ready;
- the managed environment refuses another environment's grants and sessions;
- gross, credit and net costs can be reconstructed for each activation; and
- the same application and contract versions pass local and hosted checks, with
  enforcement differences documented.

Review the exact service bindings in M3.2, the first measured activation after
M3.3 and the cost target after several representative demonstrations. Revisit
this decision if teardown time prevents useful demonstrations, residual cost
dominates the target, the managed-trust gate cannot be satisfied or another
provider offers materially better evidence without weakening portability.

# Backend workspace

The backend is a Rust workspace for the framework host and for logical
component packages that have reached implementation. A package boundary does
not imply a separately deployed service.

The workspace contains:

- `apps/framework-host`: the local command/outcome adapter, component
  description, capability manifest and safe health boundary;
- `apps/m3-runtime`: one image/runtime entry point with separate Scenario
  Director and Presentation Gateway process modes;
- `crates/contracts`: shared Rust representations of the canonical common
  and M2 identity/authorisation schemas;
- `crates/core`: shared identifiers and component descriptors; and
- `components/aut-01`: the bounded, replaceable M2 policy-decision adapter;
- `components/iam-01`: the M2 local-synthetic trust, workload context, grant
  and synthetic-session reference implementation; and
- `components/int-01`: the in-development M1 contract, idempotency, journal and
  outcome reference implementation plus the M3.3 NATS JetStream adapter;
- `components/ctl-01`: the M3.3 Scenario Director package, lifecycle,
  controlled-time, reset and checkpoint reference implementation; and
- `components/ctl-02`: the M3.3 presentation manifest, registration, cue,
  outcome, reset and bounded-fault reference implementation.

The M1 and M2 implementations have no listener. Their locked append journals
are qualified only as single-host assurance state; they are not an event
transport, business store, distributed session service or production audit
system. M2 generates an environment-specific local signing key and establishes
safe synthetic-session references, but has no browser login, managed issuer or
external-human identity integration. cREXX remains excluded while its future
`RUL-01` boundary is designed.

M3.3 adds a loopback/port-forwarded HTTP and SSE adapter plus TLS/NKey NATS
JetStream for a closed synthetic assurance package. `CTL-01` and `CTL-02` own
separate SQLite databases and run as separate workloads. This remains a
single-instance development binding, not a managed-identity, high-availability,
business-data or production service.

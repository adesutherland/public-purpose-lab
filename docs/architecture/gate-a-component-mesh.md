# Gate A component mesh

Status: Implementation baseline

Date: 1 September 2026

Information profile: Synthetic only

## Purpose

Gate A makes the founder-approved deployable component baseline concrete. It
proves that the complete initial mesh can run as separately identified
workloads, authenticate to the event infrastructure, report honest readiness
and exchange a bounded command and conclusive outcome. It does not implement
the later business journeys.

## Deployed shape

The local Compose profile uses one image containing two Rust binaries. The M3
runtime continues to run `scenario-director`, `presentation-gateway` and
`identity-broker`. Nine instances of the configurable component host run the
remaining responsibilities. NATS is the shared authenticated event carriage;
it is not a shared business-state owner.

```text
Director (CTL-01) -----------+
Presentation (CTL-02) -------+--> authenticated readiness events --+
Identity (IAM-01) -----------+                                      |
                                                                    v
AUT / DOM / CNT / KNO / WRK / RPT / AUD / OPS / INT instances --> NATS
       ^                                                            |
       +---------- bounded capability commands from OPS-01 <--------+
                                                                    |
Operations Console <------ real readiness and outcome projection <--+
```

Each of the twelve instances has a distinct component ID, instance ID and
environment-generated NKey. The local synthetic root and private credentials
remain within the environment setup directory; only the intended credential
is mounted into each workload.

## HTTP contracts

Every configurable host exposes:

| Method | Path                | Gate A meaning                                             |
| ------ | ------------------- | ---------------------------------------------------------- |
| `GET`  | `/health/live`      | Process is live; it is not a business-completion claim.    |
| `GET`  | `/health/ready`     | Configuration and event connection completed successfully. |
| `GET`  | `/health/contracts` | Working contract versions understood by the host.          |
| `GET`  | `/api/v1/component` | Component, instance, capability and provenance summary.    |

`OPS-01` additionally exposes:

| Method | Path             | Gate A meaning                                                       |
| ------ | ---------------- | -------------------------------------------------------------------- |
| `GET`  | `/api/v1/mesh`   | Projection of expected instances against actually observed events.   |
| `GET`  | `/api/v1/events` | Privacy-minimised recent operational events.                         |
| `POST` | `/api/v1/probe`  | Issues one bounded capability command to each configurable instance. |

## Working event binding

The Gate A `O-001` v0.1.0 binding is an implementation working draft, not the
canonical logical health-and-readiness contract. A command includes contract
and version, command and target, issuer, environment, purpose, correlation,
causation, idempotency and time. A readiness or outcome event includes the
component and instance, workload identity, environment, status, capability,
source and image provenance, time and relevant command references.

Subjects are confined to:

- `ppl.gate-a.commands.<component-id>` for bounded probes; and
- `ppl.gate-a.events.<component-id>` for readiness and conclusive outcomes.

Every receiver validates contract version, target, environment, issuer,
purpose, supported command and idempotency before accepting a probe. Exact
redelivery returns a duplicate outcome; changed content under the same key is
refused. A Gate A probe changes no business state.

## Operations projection

The Operations Console maintains an expected catalogue of twelve components
but derives `ready` only from received readiness events. An unobserved instance
is `missing`; an instance whose event is more than fifteen seconds old is
`stale`. Normal readiness repeats every five seconds, allowing the projection
to recover after restart without inventing prior activity.

The timeline is deliberately privacy-minimised and transient in Gate A. It
contains identifiers, outcomes and timings but not credentials, private keys,
source bodies or protected content. Durable audit reconstruction remains a
later functional gate.

## Acceptance evidence

Gate A is ready for founder review only when the automated smoke and human
walkthrough show:

1. all twelve expected components ready with twelve distinct workload
   identities;
2. the Operations Console refreshing from real readiness events;
3. nine bounded capability commands and nine conclusive correlated outcomes;
4. safe recovery after broker and workload restart;
5. the existing Director, Presentation and Workbench surfaces remain usable;
6. real screenshots and the Gate A progress/show-and-tell PDF identify the
   exact revision and environment; and
7. later business capabilities are clearly labelled as not yet implemented.

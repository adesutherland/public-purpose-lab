# Gate B environment, identity and portal orchestration

Status: Implemented baseline; founder acceptance and visual closure evidence pending

Date: 1 September 2026

Information profile: Synthetic only

## Purpose

Gate B implements the approved `DS-01` and `DS-02` journeys. It lets an
authenticated presenter inspect the active environment, select an admitted
demonstration, establish an environment- and session-bound synthetic reviewer,
and move Presentation and Workbench through semantic views. It uses the Gate A
component mesh without adding a source-processing or reporting claim.

The Gate B implementation is the current baseline, not an immutable design.
Later gates may revise mechanisms when executable evidence justifies the
change, while retaining the approved security and authority invariants.

## Executable shape

```text
external presenter
      |
      v
Director / CTL-01 -- D-002 lifecycle ----> Director state
      |                    |
      |                    +-- I-004 synthetic sign-in request --> IAM-01
      |                                                        --> Workbench binding
      |
      +-- P-003 semantic cue --> authenticated NATS --> CTL-02 / target session
                                                         |
                               Presentation or Workbench resolves its own view
                                                         |
                              P-004 outcome + O-001 operational events
                                                         |
                                                         v
                                               OPS-01 event timeline
```

The Director, Presentation Gateway, Identity Broker, Operations Console and
authenticated NATS infrastructure remain separate deployed workloads. The
Presentation and Workbench frontends currently share the Presentation Gateway
backend and image, but have distinct paths, registrations, roles and browser
sessions. This is physical co-location, not shared logical ownership.

## Actor and session model

| Actor or identity         | Gate B responsibility                                               | Boundary                                                                          |
| ------------------------- | ------------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| External presenter        | Signs into Director, selects and controls the demonstration         | May coordinate presentation; cannot act as a reviewer or create business facts    |
| External surface operator | Opens and registers Presentation or Workbench                       | Holds only that surface's opaque application session and CSRF value               |
| `synthetic-reviewer`      | Demonstrates the reviewer role inside Workbench                     | Established by a backend-only, environment- and Demonstration Session-bound grant |
| Workload identities       | Authenticate Director, Gateway, Identity and Operations event paths | Separate environment-generated credentials and least-privilege subjects           |

An external surface operator and a synthetic application actor may coexist in
one Workbench session because they represent different responsibilities. A
synthetic actor on another surface is a separate application session. Stopping
or resetting the Demonstration Session terminates its synthetic bindings but
does not sign out the external presenter or surface operator.

Private keys, signed grants, broker credentials and usable synthetic-session
references do not enter browser state, events, URLs or diagnostic output.

## Surface responsibilities

| Surface            | Added Gate B views and functions                                                 | Explicit boundary                                                                      |
| ------------------ | -------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| `DIR-ENVIRONMENT`  | Presenter sign-in, environment ID, runtime and trust profile, Operations link    | Readiness is an observed technical condition, not compliance evidence                  |
| `DIR-CATALOGUE`    | One admitted scenario with purpose, actors, dependencies, status and limitations | Catalogue admission creates neither a Demonstration Session nor a synthetic session    |
| `DIR-RUN`          | Prepare, register, assign reviewer, start, cue, pause, resume, stop and reset    | Director coordinates but does not own portal routes or business state                  |
| `PRES-INTRO`       | Problem, synthetic organisation, actors, intended outcome and limitations        | A displayed view establishes presentation progress only                                |
| `WB-ENGAGEMENT`    | Bounded synthetic engagement context and assigned-reviewer banner                | No engagement record is created in Gate B                                              |
| `WB-SOURCE-INTAKE` | Local upload, link, paste and rights-entry controls                              | Submission is disabled; no content is transmitted, persisted, quarantined or processed |
| `OPS-EVENTS`       | Correlated Director, surface, identity and view-request/outcome events           | Transient operational evidence is not durable audit evidence                           |

Workbench also provides ordinary accessible navigation between its admitted
views. That navigation is intentionally local to the application and emits no
business event. Director links are operator conveniences configured for each
environment; presentation control itself never depends on a URL or DOM action.

## Contracts exercised

| Contract             | Gate B use                                                                                                                     |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| `D-001`              | Admits package `presentation-control-assurance` v1.2.0 with the two approved stages and four semantic views                    |
| `D-002`              | Creates and controls a revision-checked Demonstration Session                                                                  |
| `D-003`              | Sets bounded logical time and retains the existing scoped reset/fault controls                                                 |
| `D-004`              | Distinguishes presentation checkpoints from business completion                                                                |
| `P-001`              | Admits Presentation Gateway manifest v1.2.0 and the `pres-*` and `wb-*` view vocabulary                                        |
| `P-002`              | Binds one current, role-specific surface registration to a session and connection generation                                   |
| `P-003`              | Carries bounded semantic view, text context and opaque correlation—not a route, credential or command to mutate business state |
| `P-004`              | Records applied, refused, unsupported, expired or failed target-owned outcomes with `businessCompletionClaimed=false`          |
| `I-004` / `I-005`    | Requests and establishes the backend-only synthetic reviewer binding                                                           |
| working `O-001` v0.1 | Publishes privacy-minimised operational facts for the Gate B journey                                                           |

The HTTP and SSE paths are physical adapters. The semantic contracts and
events remain the interoperable boundary.

## Functional rules

1. The catalogue reports `ready` only when the authenticated event path needed
   for the scenario is available; it retains explicit limitations.
2. Creating a Demonstration Session does not create a synthetic application
   session.
3. A surface must register the appropriate role and admitted view set before a
   cue can be issued.
4. The reviewer grant is accepted only by the target Workbench registration in
   the same environment and Demonstration Session.
5. Each target resolves its own semantic view and returns the conclusive
   outcome. Appearance is never inferred merely because a cue was sent.
6. Unsupported views are refused before delivery; expired, stale or mismatched
   target outcomes are refused by the existing presentation controls.
7. Pause changes Director sequencing only. Stop and reset terminate synthetic
   bindings; reset creates a distinct successor session.
8. Presentation outcome cannot create engagement, source, workflow, knowledge
   or report state.
9. Gate B operational events carry correlation and component ownership but no
   source bodies, credentials, grants or usable session values.

## Portable binding

Local native, container and Minikube profiles default the Director's operator
links to loopback ports. Hosted profiles expose no local fallback. A deployment
may provide `PPL_PRESENTATION_SURFACE_URL`, `PPL_WORKBENCH_SURFACE_URL` and
`PPL_OPERATIONS_SURFACE_URL`; these are navigation bindings only and do not
change the event-based control contract.

Local profiles show the environment-local synthetic-root classification.
Hosted profiles require the managed real-root and OIDC bindings already
accepted for M3.4. Gate B does not widen the authorised information profile or
qualify the hosted environment for real data.

## Failure and recovery position

- Missing event or identity infrastructure makes the catalogue unavailable or
  causes the protected action to fail closed.
- A surface registered to another session, role or generation cannot consume
  the cue or synthetic reviewer binding.
- Browser refresh restores external and synthetic context from backend-owned
  state without exposing grant material.
- Duplicate, delayed, expired and unsupported cues retain the M3.3/M3.4
  idempotency and refusal behaviour.
- Operations publication failure is logged but cannot turn a refused or failed
  control operation into success.

## Gate boundary and closure

Gate B establishes visible DS-01/DS-02 orchestration. It does not establish an
engagement record, source acquisition, quarantine, validation, staging,
knowledge processing, durable audit reconstruction, human review or reporting.

Closure requires the automated checks and a real browser walkthrough of the
approved flow, retained screenshots from the exact build, the visually checked
show-and-tell PDF and founder acceptance. The following Gate C implements the
first business-process path: governed source intake and visible processing.

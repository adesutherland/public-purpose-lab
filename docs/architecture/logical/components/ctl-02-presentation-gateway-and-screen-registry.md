# CTL-02: Presentation Gateway and screen registry

Status: Accepted M3.2 logical baseline; implementation not started

Version: 0.1.0

Last reviewed: 27 August 2026

Governing decisions:
[ADR-0013](../../decisions/0013-use-nats-jetstream-for-the-first-component-event-binding.md),
[ADR-0014](../../decisions/0014-bind-presenters-through-google-oidc-and-backend-sessions.md),
[ADR-0015](../../decisions/0015-use-a-backend-mediated-presentation-channel.md) and
[ADR-0016](../../decisions/0016-bind-the-first-hosted-preview-to-gke-autopilot.md)

## Purpose

`CTL-02` makes presentation surfaces discoverable and controllable through
semantic contracts rather than browser links, URLs or remote-control scripts.
It accepts surface capabilities, binds an authenticated surface to one
Demonstration Session and declared surface slot, routes short-lived cues and
records safe delivery outcomes.

It is a presentation-control component. It is not an identity provider,
application-session owner, browser router, business-command gateway or source of
evidence that a business outcome occurred.

## Accountable ownership

The Presentation Gateway owner is accountable for:

- admission and versioning of
  [`P-001`](../contracts/presentation/p-001-presentation-capability-manifest.md)
  capability manifests;
- one authoritative, revisioned
  [`P-002`](../contracts/presentation/p-002-presentation-surface-registration.md)
  binding for each active Demonstration Session surface slot;
- cue target validation, bounded delivery, expiry and reconciliation under
  [`P-003`](../contracts/presentation/p-003-presentation-cue.md);
- attributable
  [`P-004`](../contracts/presentation/p-004-presentation-cue-outcome.md)
  outcomes without inferring business completion;
- explicit supersession, disconnect, expiry and revocation behaviour; and
- privacy-minimised presentation evidence and safe operational status.

The scenario package declares required surface roles and capabilities.
`CTL-01` owns scenario progression. The application backend authenticates its
surface and owns any application session. `CTL-02` validates only the
presentation binding and delivery operation it owns.

## Non-responsibilities

`CTL-02` does not:

- authenticate an external human directly, issue trust or establish an
  application session;
- validate, consume, expose or forward an `I-004` signed sign-in grant;
- authorise a business command or convert a cue into one;
- hold browser cookies, access tokens, private keys or reusable surface
  credentials;
- store or transmit browser routes, URLs, DOM selectors or window handles as
  cue meaning;
- allow a browser to connect directly to the component event broker;
- assert that a view was visible, understood or acted upon merely because it
  was applied by a surface;
- make one surface registration valid in another environment or Demonstration
  Session; or
- claim production, compliance, legal, clinical or professional authority.

## Principals and decision rights

| Principal or role               | Decision right and limit                                                                                                          |
| ------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| Authorised presenter            | Operates the Director through `CTL-01`; has no direct registry or broker authority.                                               |
| Scenario Director workload      | May request an authorised cue for the current session and target slot; cannot register a surface or choose a browser route.       |
| Presentation Gateway workload   | May validate registrations, resolve current bindings, route cues and record outcomes within its contract authority.               |
| Application backend workload    | Authenticates its browser/session, attests its application binding and registers only supported presentation capabilities.        |
| Presentation Surface            | Resolves an accepted semantic view locally and returns a bounded outcome; cannot receive business or identity authority in a cue. |
| `IAM-01` and target application | Own synthetic grant validation and application-session establishment; expose only a redacted `I-005` outcome.                     |
| Service owner or operator       | Approves surface applications and performs named recovery actions; operator access is not presenter or synthetic-user authority.  |

Network location, a Kubernetes service account name, possession of a frontend
bundle or an open event stream is not sufficient authority. Workload identity,
human/application session context, current policy and the registered binding
are checked independently where applicable.

## Surface capability model

`P-001` specialises `C-005` for a presentation-capable application release. A
manifest declares stable semantic view identifiers, accepted context types,
contract versions, accessibility characteristics and bounded rendering
constraints. It never declares routes or credentials.

Capabilities describe meaning such as `scenario.overview`,
`identity.synthetic-user` or `evidence.checkpoint`, not implementation paths.
The Presentation Surface owns the local mapping from an accepted semantic view
to its components and route state. Changing that internal mapping does not
change a scenario package when the semantic contract remains compatible.

Manifest admission is descriptive. It does not register a live surface,
authorise a cue or prove that the application is healthy.

## Surface-slot and registration model

A scenario package declares one or more presentation surface roles. At runtime,
each required occurrence becomes a **surface slot** scoped to one environment,
package version and Demonstration Session. Examples include `public-portal`,
`staff-workbench` and `audience-display`.

The first profile permits one authoritative active registration for each
surface slot. A package may declare several slots, including more than one slot
with the same role when the multiplicity is explicit. This is not a
scenario-wide one-screen restriction.

Each `P-002` registration binds:

- one environment, trust profile and Demonstration Session;
- one package-declared surface slot and role;
- one application and application release;
- one authenticated application-backend workload;
- one opaque surface instance and current connection generation;
- one accepted `P-001` manifest and capability digest;
- where relevant, one redacted external or synthetic application-session
  reference; and
- an issue time, lease expiry, revision, status and safe evidence references.

A registration is not an application credential. A browser receives only the
ordinary application session material owned by its backend and the minimum
non-secret presentation state needed to render or report an outcome.

## Registration lifecycle

```text
unregistered -> registering -> active -> disconnected -> active (new generation)
                         |          |           |
                         |          |           +-> expired or superseded
                         |          +-> revoked, expired or superseded
                         +-> refused or failed
```

- `registering` is not cue-ready.
- `active` means the binding and delivery channel are current; it is not proof
  that the browser is visible to an audience.
- `disconnected` retains a short reconciliation window but receives no new cue.
- reconnect creates a new connection generation under the same registration or
  a new registration according to current policy.
- supersession makes the previous generation unable to receive or conclude a
  later cue.
- session stop, reset, trust change, application-session termination or an
  authorised owner action may revoke or expire the binding.

Lease and expiry decisions use protected operational time. Scenario logical
time cannot extend a registration or reconnect window.

## Cue routing and authority

`CTL-01` sends `P-003` to `CTL-02`, naming the current Demonstration Session,
surface slot, semantic view, bounded context and expiry. `CTL-02`:

1. validates envelope, requester, session, expected revisions, purpose,
   idempotency and expiry;
2. resolves the one current registration for the target slot;
3. confirms that its accepted `P-001` supports the semantic view and context;
4. records the attempt before delivery;
5. sends the cue through the protected application-backend channel; and
6. validates and records the resulting `P-004` or explicit timeout/uncertainty.

The cue contains no route. The Presentation Surface maps the semantic view to
its own current implementation and refuses an unknown, incompatible or unsafe
request.

A cue can change only presentation state. If a scenario needs an application
business action, `CTL-01` sends a separate command to the component that owns
that action with its own authority and outcome. Cue ordering or visual success
cannot substitute for that command or fact.

## Repetition, ordering and reconnect

- Every cue has an idempotency key and immutable content digest scoped to its
  environment, Demonstration Session and target surface slot.
- Repeating the same cue returns or completes the original safe result;
  reusing its identity for different content is refused.
- Delivery is at least once at the component transport boundary. The gateway
  and surface deduplicate by cue identity and registration generation.
- Ordering is per surface slot and registration revision; no global order is
  assumed across surfaces.
- A cue is applied only when the target session, slot, registration revision,
  connection generation and protected expiry are current.
- A reconnect does not replay a queue. It may reconcile only the latest still-
  valid cue explicitly recorded for that slot and generation.
- A successful scenario reset creates a new Demonstration Session. No prior
  registration, cue, idempotency scope or delayed outcome transfers to the
  successor.

## Outcome meaning

`P-004` distinguishes `applied`, `refused`, `unsupported`, `expired`,
`duplicate`, `superseded`, `failed` and `uncertain` outcomes.

`applied` means that the authenticated current surface accepted and committed
the semantic presentation state for the named cue. It does not mean:

- a human saw or understood the view;
- a browser URL was opened successfully;
- an application session was created;
- a business command or workflow step completed; or
- a compliance, legal, clinical or professional condition was satisfied.

`D-004` may use an attributable `P-004` as a presentation-progress observation
only. Business and evidential checkpoints require their declared authoritative
sources.

## Failure, restart and recovery

Registry state and conclusive cue outcomes survive an ordinary `CTL-02`
restart. Recovery classifies each incomplete delivery as not delivered,
conclusively decided or uncertain. The gateway reconciles the original cue and
registration generation before any retry; it does not infer success from the
current page or blindly replay work.

A lost browser connection makes the slot unavailable for new delivery. A new
generation may register after current authentication and policy checks. A late
outcome from the old generation is retained as stale evidence but cannot
conclude the new generation's cue.

Missing or corrupt registry state makes affected slots not ready. Recovery does
not invent a binding from broker subscriptions, browser state, logs or operator
assertion. Where security or session continuity cannot be proven, the
application session is terminated or left unavailable under its owner's
policy.

## Audit, privacy and observability

Safe evidence records:

- environment, package and Demonstration Session references;
- surface slot, role, application, release, manifest digest and capability
  version;
- safe principal and registration references, revision and connection
  generation;
- cue identity, semantic view, outcome, protected times and safe reason;
- correlation, causation, idempotency, policy and evidence references; and
- disconnect, supersession, expiry and recovery ownership.

It excludes raw Google or other identity assertions, signed grants, tokens,
cookies, private keys, browser routes, URLs, DOM details, event-broker
credentials and reusable channel values. Presenter-facing reasons may be less
detailed than authorised support evidence to prevent enumeration.

Operational views must show disconnected, stale, expiring, superseded and
uncertain bindings distinctly. They must not describe `local-synthetic` trust
as managed or hide the active trust profile.

## Deployment and replaceability

`CTL-02` may initially share one Rust deployable and durable store with
`CTL-01`, provided their logical state, authorisation checks and contract
evidence remain distinguishable. The TypeScript `UX-04` application uses an
application backend or backend-for-frontend as the protected channel endpoint;
the browser never connects directly to the component event broker.

The event broker, server-to-browser protocol, backend API framework, state
store, ingress, identity provider and Kubernetes distribution are replaceable
bindings. The accepted M3.2 ADRs select the first local, Minikube and Google
Cloud profiles without claiming implementation or qualification.

Demonstrator evidence maps the component responsibility, contract behaviour,
failure modes and operating lessons back to the Architecture Portal logical
blueprint. That feedback is a portable architecture result, not a claim that
the Portal already implements or operates this component.

## Dependencies and contract relationships

`CTL-02` depends on:

- `C-001` to `C-006` and `INT-01` for envelope, compatibility and delivery
  semantics;
- `D-001`, `D-002` and `CTL-01` for admitted surface slots and the current
  Demonstration Session;
- `I-001`, `I-002`, `I-004`, `I-005`, `IAM-01` and `AUT-01` for distinct human,
  workload, synthetic-session and authorisation responsibilities;
- `P-001` to `P-004` for presentation capability, registration, cue and
  outcome semantics; and
- `O-001`, `C-004` and later `AU-001` for readiness, evidence and audit.

## Conformance evidence required before implementation acceptance

Evidence must show that:

1. an unauthenticated backend, unsupported manifest or undeclared application
   cannot register a surface;
2. a surface cannot register into another environment, session or slot;
3. the current registration supersedes prior generations without cross-
   delivery;
4. duplicate, changed-content, delayed, expired and out-of-order cues fail or
   reconcile as specified;
5. reconnect recovers only an explicitly current, unexpired cue and never
   replays a prior-session cue;
6. a late outcome from a disconnected or superseded generation cannot conclude
   current work;
7. cues contain no URLs or routes and cannot invoke business behaviour;
8. browser code has no direct broker, signing, grant-validation or private-
   store access;
9. an applied cue cannot satisfy a business checkpoint;
10. gateway restart cannot duplicate or invent an applied outcome;
11. registration and cue expiry use protected operational time, not scenario
    logical time; and
12. events, evidence, logs, browser history and diagnostics contain no usable
    credential, grant, session value, route or broker secret.

## Current limitations and decisions deferred

`CTL-02` and `P-001` to `P-004` are accepted logical specifications. No schema,
runtime, database or transport is implemented. Exact size, rate, duration,
connection, accessibility and multi-surface limits require executable evidence.

ADR-0013 to ADR-0016 accept the first product bindings at design maturity only.
Acceptance does not claim that NATS JetStream, Google OIDC, server-sent events,
GKE Autopilot, Cloud KMS or OpenTofu is deployed, integrated or qualified.

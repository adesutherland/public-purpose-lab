# ADR-0015: Use a backend-mediated presentation channel

Status: Accepted
Date: 2026-08-27

## Context

The current experience shows that directing browsers with frontend links,
URLs, window references and remote navigation is fragile. `P-001` to `P-004`
replace that mechanism with semantic views and authenticated surface bindings.

The browser still needs a small delivery channel from its own application
backend. Direct NATS access would give frontend code infrastructure
credentials, subject visibility and replay behaviour outside the application
session boundary. A bidirectional socket is possible, but M3 needs primarily
server-to-browser cues and a bounded browser-to-server outcome.

## Decision

Use a same-origin application backend or backend-for-frontend for every
presentation surface. Bind the browser channel as:

- **server-sent events (SSE)** from the application backend to the browser for
  semantic presentation cues and safe connection state; and
- an authenticated, CSRF-protected **HTTP POST** from the browser to its
  backend for `P-004` application outcomes.

The application backend is the component-bus participant. It registers the
surface with `CTL-02`, receives the authorised `P-003` through the `INT-01`
adapter, exposes only the bounded semantic cue to its current browser session,
validates the outcome and publishes the safe `P-004`. The browser has no NATS,
KMS, sign-in-grant, private-store or component-service credential.

The SSE and POST endpoints are ordinary deployment routes protected by the
application session. They are never present in scenario packages, capability
manifests, cues or evidence, and are not the semantic meaning of a view.

The first connection profile requires:

- HTTPS outside loopback development, secure application cookies and strict
  origin policy;
- no state mutation on the SSE GET;
- CSRF and origin validation on outcome POST;
- an opaque, non-authority SSE event identifier scoped to one registration
  generation;
- backend revalidation of application session, Demonstration Session,
  registration generation and lease on initial connection and reconnect;
- a bounded heartbeat and idle timeout for connection state only;
- no generic event replay from a browser-supplied `Last-Event-ID`; and
- reconciliation of at most the latest explicitly retained, still-valid cue
  for the current surface slot and generation.

The browser resolves the semantic view against the accepted `P-001` manifest.
It commits the new presentation state before returning `applied`. Unknown,
expired, incompatible, superseded or unsafe cues return the corresponding
`P-004` result.

SSE delivery, HTTP success and browser connection state are transport facts.
They do not mean the cue was applied, a person saw it or a business action
completed.

## Alternatives considered

- **Direct links, routes or browser automation:** rejected because they couple
  the scenario to implementation detail and have already proved fragile.
- **Direct browser NATS/WebSocket:** rejected because it exposes broker
  credentials and bypasses the application backend/session boundary.
- **Application WebSocket:** viable if later interactions need sustained
  bidirectional low-latency traffic, but adds connection and message-protocol
  complexity before M3 needs it.
- **Long polling:** simple but creates repeated requests and less clear
  reconnect semantics for an event-driven demonstration.
- **WebRTC or browser-to-browser control:** unnecessary and expands network,
  identity and traversal risks.

## Consequences

- The browser channel is simple, standards-based and separate from component
  event transport.
- Each presentation-capable application needs a small backend or BFF even when
  its frontend is otherwise static.
- Cookie, CSRF, origin, reconnect and proxy buffering/timeout configuration
  require explicit tests in local and hosted profiles.
- SSE is one-way; any future high-rate or genuinely bidirectional use may
  justify a compatible WebSocket binding through a new ADR.
- Internal routes remain replaceable implementation detail and can differ
  across Mac, Linux, Windows, Minikube and GKE.

## Validation and review

Evidence must demonstrate:

- authorised delivery and outcome through the application backend;
- direct browser broker access denied and no broker code/credential in the
  frontend bundle;
- wrong origin, missing CSRF, expired application session and wrong
  registration generation refusal;
- disconnect, network interruption, backend restart and browser refresh;
- copied or stale `Last-Event-ID` cannot cross a generation or session;
- latest-valid-cue reconciliation without historical replay;
- proxy/ingress streaming behaviour, idle timeout and resource bounds;
- semantic resolution with no URL or DOM selector in contract evidence; and
- an applied presentation outcome remaining distinct from business completion.

Review SSE if measurements show proxy incompatibility, excessive connection
cost, multi-tab ambiguity or a demonstrated bidirectional requirement.

## Reference material

- [WHATWG server-sent events standard](https://html.spec.whatwg.org/multipage/server-sent-events.html)

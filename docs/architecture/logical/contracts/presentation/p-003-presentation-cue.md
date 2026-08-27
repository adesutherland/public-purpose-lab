# P-003: Presentation cue

Status: Accepted M3.2 logical baseline; schema and implementation pending

Version: 0.1.0

Last reviewed: 27 August 2026

Owner: [`CTL-01`](../../components/ctl-01-scenario-director.md) owns the
scenario request;
[`CTL-02`](../../components/ctl-02-presentation-gateway-and-screen-registry.md)
owns target validation and delivery

Semantic type: short-lived presentation-control command

Canonical schema: Not selected in M3.2

## Purpose

`P-003` asks one currently registered surface slot to apply a semantic view
with bounded presentation context. It replaces fragile URL, link, browser-
window and DOM control while carrying no business, identity or session-
establishment authority.

## Participants and authority

| Role           | Participant             | Responsibility                                                                      |
| -------------- | ----------------------- | ----------------------------------------------------------------------------------- |
| Cue requester  | `CTL-01`                | Requests an admitted semantic view for the current scenario stage and surface slot. |
| Delivery owner | `CTL-02`                | Validates current binding/capability, records the attempt and mediates delivery.    |
| View resolver  | `UX-04` application     | Resolves the semantic view locally and commits or refuses presentation state.       |
| Business owner | Target domain component | Receives any separate business command; it is never invoked by interpreting a cue.  |

Presenter authority is exercised through an authorised `D-002` or other
explicit Director action. Possession of cue content, a browser or an event-
channel connection is not authority to issue a cue.

## Cue information

A cue contains:

- cue identifier, contract/version, immutable content digest and idempotency
  key;
- environment, package and Demonstration Session identifiers and expected
  session revision;
- target surface slot and expected registration revision/generation;
- semantic view identifier and compatible capability version;
- bounded, schema-declared presentation context or safe `C-004` references;
- presenter, Director workload, purpose and classification context through
  `C-001` and `C-002`;
- issue and expiry times based on protected operational time; and
- correlation, causation, stage/step and safe evidence references.

Context may select a synthetic/public record or explain an already authorised
outcome. It cannot contain an instruction that the surface interprets as a
business mutation.

## Prohibited content and behaviour

A cue is refused if it contains or requires:

- a URL, browser route, host, query string, fragment, window identifier or DOM
  selector;
- a token, cookie, signed grant, private key, session value or broker
  credential;
- shell, SQL, script, executable expression or arbitrary component command;
- an undeclared context field or substantive protected content outside the
  environment profile;
- a target not declared by the package and current registration; or
- a request to infer identity, authorisation, business completion or human
  attention from presentation state.

The frontend's internal route may change as an implementation detail. It is not
reported back as contract evidence or stored in the cue history.

## Validation and delivery

Before delivery, `CTL-02` confirms:

1. supported envelope and exact cue version;
2. authenticated, authorised requester and permitted purpose;
3. current environment, package, session and expected revision;
4. current target registration, connection generation and surface slot;
5. admitted `P-001` support for the view and context;
6. cue issue/expiry, size and rate bounds;
7. idempotency identity and immutable content; and
8. absence of prohibited content.

The Presentation Surface repeats current session, generation, expiry,
capability and context validation before applying the view. Transport
acceptance is not presentation application.

## Delivery, ordering and repetition

- Delivery may be at least once. The surface commits each cue identity at most
  once for one registration generation.
- An identical duplicate returns the prior `P-004` result or a `duplicate`
  outcome without reapplying side effects.
- Reuse of a cue identity or idempotency key for different semantic content is
  refused.
- Ordering is defined only within the target surface slot and registration
  revision. A later accepted cue may supersede an earlier unapplied cue.
- Expiry is checked before dispatch and before application. Queue delay,
  reconnect or scenario logical time never extends validity.
- Reconnect may reconcile only the latest applicable, unexpired cue explicitly
  retained for the current slot. There is no general historical replay.
- Cues from a stopped, failed, completed or superseded session cannot be
  applied unless that terminal state's contract explicitly permits the
  presentation-only operation.

## Refusal and failure

Safe reason classes include `requester-not-authorised`, `session-inactive`,
`session-revision-stale`, `surface-unregistered`,
`registration-generation-stale`, `view-unsupported`, `context-invalid`,
`prohibited-content`, `expired`, `duplicate-content-conflict`,
`rate-or-size-limit`, `delivery-unavailable` and `recovery-required`.

Loss of acknowledgement after possible application is uncertain until
reconciled through `P-004`; the cue is not blindly replaced with a new identity.

## Evidence and checkpoint meaning

Cue evidence records only safe semantic and binding references, times,
decision, delivery state, correlation and the `P-004` reference. It excludes
routes, credentials, raw application records and protected content not needed
to explain the presentation operation.

A cue or its transport acknowledgement cannot satisfy a `D-004` business
checkpoint. Only a conclusive `P-004` may satisfy a declared
`presentation-progress` observation.

## Conformance evidence required

Evidence must exercise authorised, unauthorised, unsupported, duplicate,
changed-content, delayed, expired, stale-session, stale-generation,
disconnected, reconnect and restart cases. Tests must prove route/credential
rejection and that no cue path can invoke a business mutation.

## Current limitation

This is an accepted logical specification. Canonical context schemas,
numerical bounds, transport mapping and frontend resolver API remain pending.

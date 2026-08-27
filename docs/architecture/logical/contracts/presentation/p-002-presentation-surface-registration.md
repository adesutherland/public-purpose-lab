# P-002: Presentation surface registration

Status: Accepted M3.2 logical baseline; schema and implementation pending

Version: 0.1.0

Last reviewed: 27 August 2026

Owner:
[`CTL-02`](../../components/ctl-02-presentation-gateway-and-screen-registry.md)

Semantic type: protected registration command, authoritative binding fact and
safe status outcome

Canonical schema: Not selected in M3.2

## Purpose

`P-002` binds one authenticated presentation-capable application surface to one
environment, Demonstration Session and package-declared surface slot. It makes
capability, connection generation, lease, supersession and recovery explicit
without turning the registration into an application credential.

## Participants and authority

| Role                    | Participant                            | Responsibility                                                                                         |
| ----------------------- | -------------------------------------- | ------------------------------------------------------------------------------------------------------ |
| Registration requester  | Authenticated application backend      | Attests the application/surface context and requests only its supported capabilities.                  |
| Registry owner          | `CTL-02`                               | Validates, authorises and owns the authoritative revisioned binding.                                   |
| Session authority       | `CTL-01`                               | Supplies current session and declared surface-slot state; cannot register a surface itself.            |
| Identity/session owners | `IAM-01`, external IdP, target backend | Supply current workload/human/session context without exposing credentials to the registry or browser. |
| Presentation client     | `UX-04`                                | Uses its ordinary application session and reports safe connection/outcome state.                       |

The browser is not trusted merely because it loaded an approved frontend. The
application backend mediates registration and the presentation channel.

## Contract variants

| Variant                           | Kind                     | Purpose                                                                          |
| --------------------------------- | ------------------------ | -------------------------------------------------------------------------------- |
| `RegisterPresentationSurface`     | Protected command        | Requests a new binding for one current surface slot.                             |
| `PresentationSurfaceRegistered`   | Authoritative fact       | Records the accepted registration, revision, generation and lease.               |
| `RefreshSurfaceRegistration`      | Protected command        | Revalidates current state and advances a bounded lease or connection generation. |
| `DisconnectPresentationSurface`   | Command or observed fact | Marks the current delivery connection unavailable without implying logout.       |
| `RevokePresentationSurface`       | Protected command        | Ends a binding under current owner, session, identity or security authority.     |
| `PresentationRegistrationStatus`  | Protected query outcome  | Returns redacted current status for Director, application or support recovery.   |
| `PresentationRegistrationRefused` | `C-003` outcome          | Reports safe validation or policy refusal.                                       |

## Binding identity

An accepted registration records:

- registration identifier, revision and connection generation;
- environment, trust profile, package and Demonstration Session;
- package-declared surface-slot identifier and role;
- application, application release and authenticated backend workload;
- opaque surface-instance reference;
- accepted `P-001` manifest identifier, version and digest;
- supported semantic-view identifiers and compatibility ranges;
- optional redacted external or synthetic application-session reference;
- issue, last-seen, lease-expiry and terminal times from protected operational
  clocks;
- state, authority/policy, correlation, idempotency and evidence references;
  and
- safe reconnect and supersession metadata.

It contains no cookie, bearer token, signed grant, private key, route, URL,
broker credential or reusable presentation-channel secret.

A synthetic sign-in surface first registers from its authenticated application
backend in `pre-session` mode. That current workload-bound registration is the
surface authority input for `I-004`; it does not imply that a synthetic human is
already logged in. After the target application and `IAM-01` produce an
`I-005` established outcome, a revisioned refresh may attach the redacted
application-session reference and enable views that require that user context.
The raw grant never crosses this contract or reaches the browser.

An externally authenticated application may instead register with its current
backend session context. In both cases, `CTL-02` records whether the binding is
`pre-session`, `external-session` or `synthetic-session`; it does not validate
the external assertion or synthetic grant itself.

## Preconditions and validation

Registration requires:

1. an admitted current `D-001` package and active registrable Demonstration
   Session;
2. a declared, currently vacant or explicitly supersedable surface slot;
3. authenticated `I-002` application-backend workload identity;
4. current application/human session context when the surface requires it;
5. an admitted `P-001` manifest matching the application release;
6. compatible environment, trust, information and contract profiles;
7. an `AUT-01` permit where policy requires it, with obligations enforced; and
8. durable registry state and protected time sufficient for the lease.

An external Google or other identity assertion establishes identity only. It
does not select a scenario, slot or presenter role. A synthetic `I-005` outcome
establishes a target application session only. `CTL-02` independently binds the
current surface to the declared slot.

## Slot uniqueness and multiplicity

The first profile permits at most one authoritative `active` registration per
environment, Demonstration Session and surface slot. A scenario may contain
many slots and therefore many simultaneous external or synthetic users and
surfaces.

Reconnect may advance the connection generation of the current registration.
A deliberate replacement creates a new registration or revision and
atomically supersedes the old authority. The old generation immediately loses
delivery and outcome authority even if its network connection remains open.

## Lease, reconnect and terminal behaviour

Registrations are short-leased and revalidated against current session,
workload, application-session, manifest and policy state. A heartbeat alone
cannot extend a registration whose authority is no longer valid.

Reconnect supplies the existing safe registration reference and proves the
current application/session context through the backend. It never accepts a
browser-asserted generation or restores a binding from local storage alone.

Stop, reset, session supersession, trust change, application-session
termination, manifest withdrawal or authorised revocation makes the
registration unavailable according to current policy. A successful reset
requires a new binding to the successor Demonstration Session even when role,
display name and application are unchanged.

## Refusal and failure

Safe reason classes include `session-not-registrable`, `slot-not-declared`,
`slot-already-bound`, `workload-untrusted`, `application-session-unavailable`,
`manifest-unaccepted`, `capability-incompatible`, `profile-mismatch`,
`policy-refused`, `stale-revision`, `generation-superseded`, `lease-expired`,
`state-unavailable` and `recovery-required`.

A storage or acknowledgement failure after a possible state change is
`uncertain`. The caller reconciles the same registration operation; it does not
create a second binding blindly.

## Idempotency, ordering and evidence

- Registration commands use an idempotency key scoped to environment, session,
  slot and semantic operation.
- Repeated identical content returns the original safe result; changed content
  under the same key is refused.
- Expected revision prevents stale refresh, disconnect or revoke operations.
- No global ordering across slots is assumed.
- Evidence records safe binding identity, revisions, capability digest,
  principals, decisions, times and recovery ownership without credential or
  route material.

## Conformance evidence required

Evidence must demonstrate wrong-environment, wrong-session, undeclared-slot,
untrusted-workload, unsupported-manifest, duplicate, stale-revision,
simultaneous-registration, disconnect, reconnect, supersession, expiry, reset
and restart cases. A stale connection must be unable to receive a cue or submit
an authoritative outcome after a new generation is active.

## Current limitation

This is an accepted logical specification. Exact lease durations, rate limits,
client attestation detail and persistence await executable evidence. ADR-0015
accepts the first backend-mediated channel binding at design maturity only.

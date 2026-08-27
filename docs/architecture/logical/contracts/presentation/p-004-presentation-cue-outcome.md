# P-004: Presentation cue outcome

Status: Accepted M3.2 logical baseline; schema and implementation pending

Version: 0.1.0

Last reviewed: 27 August 2026

Owner: The registered application surface produces the result;
[`CTL-02`](../../components/ctl-02-presentation-gateway-and-screen-registry.md)
validates and records it

Semantic type: presentation-control outcome and presentation-progress fact

Canonical schema: Not selected in M3.2

## Purpose

`P-004` reports how a current registered surface handled one `P-003` cue. It
makes application, refusal, incompatibility, expiry, duplication and uncertainty
visible without exposing browser internals or asserting a business outcome.

## Participants and authority

| Role              | Participant                     | Responsibility                                                                            |
| ----------------- | ------------------------------- | ----------------------------------------------------------------------------------------- |
| Outcome producer  | Application backend and `UX-04` | Reports the result for the current authenticated surface generation.                      |
| Outcome validator | `CTL-02`                        | Confirms cue, binding, generation, timing and idempotency before recording a conclusion.  |
| Observer          | `CTL-01`                        | Uses a conclusive result only for presentation progress and recovery.                     |
| Evidence consumer | `AUD-01`, `OPS-01`              | Receives safe attributable outcome and diagnostic references under current access policy. |

A browser assertion alone is not authoritative. The application backend binds
the response to the ordinary application session and current registration
generation before `CTL-02` accepts it.

## Outcome states

| State         | Meaning                                                                                                            |
| ------------- | ------------------------------------------------------------------------------------------------------------------ |
| `applied`     | The current surface committed the semantic presentation state for this cue and generation.                         |
| `refused`     | Current policy, state, classification or safety rules did not permit application.                                  |
| `unsupported` | The current release could not resolve the semantic view or compatible context despite the requested capability.    |
| `expired`     | Protected time passed the cue or registration validity before application.                                         |
| `duplicate`   | The identical cue was already concluded; the result references the original conclusion.                            |
| `superseded`  | A newer cue, registration generation or successor session removed this cue's authority before application.         |
| `failed`      | Application was conclusively not committed because of a technical failure.                                         |
| `uncertain`   | The system cannot prove whether the surface committed the cue; recovery ownership and reconciliation are required. |

`applied` is deliberately narrow. It does not prove that a person saw,
understood or acted on the view, that a user was signed in, or that business,
workflow, evidence, legal, clinical or compliance state changed.

## Outcome information

A `P-004` outcome contains:

- outcome and source identifiers and contract versions;
- corresponding cue identity and immutable digest reference;
- environment, package, Demonstration Session and session revision;
- surface slot, registration identifier/revision and connection generation;
- application release, manifest and semantic-view references;
- outcome state, safe reason class and retry/recovery classification;
- received, attempted, applied or concluded times from protected operational
  clocks as applicable;
- source workload/application-session safe references;
- correlation, causation, idempotency, policy and evidence references; and
- optional privacy-safe renderer diagnostic code.

It contains no URL, current route, DOM content, screenshot, token, cookie,
signed grant, session value, broker credential or reusable connection value.

## Validation and conclusion

`CTL-02` accepts an outcome only when:

1. its source is the authenticated backend for the current registration;
2. cue identity and content digest match a recorded delivery attempt;
3. environment, session, slot, registration revision and generation match;
4. outcome transition and protected timing are valid;
5. reason and diagnostic fields follow the supported bounded vocabulary; and
6. the operation is not already concluded incompatibly.

A late response from a disconnected, expired or superseded generation is
recorded as stale evidence but cannot conclude the current cue. A duplicate
identical result returns the original conclusion. A conflicting duplicate is a
security or integrity failure requiring investigation.

## Recovery and retry

- `refused`, `unsupported`, `expired`, `superseded` and conclusive `failed`
  outcomes do not trigger an automatic cue with a new identity.
- `uncertain` blocks any checkpoint that requires a conclusive presentation
  result until `CTL-02` reconciles the original operation.
- Reconciliation queries the current application-backend state using the
  original cue and generation. It does not inspect or control the browser URL.
- If application cannot be proven, the result remains uncertain or is safely
  superseded; it is never promoted to applied from appearance or elapsed time.

## D-004 checkpoint use

An accepted outcome may satisfy only a checkpoint whose declared claim class
is `presentation-progress` and whose expected environment, session, slot, cue,
view, generation, source, version and freshness match.

Software health, transport delivery, synthetic sign-in, a human observation or
a domain fact requires its own contract and source. `P-004` cannot satisfy a
business or evidential checkpoint by implication.

## Evidence, privacy and analytics

Retained evidence distinguishes dispatch, backend receipt, surface attempt and
conclusive application. Presenter views may show outcome, safe reason and
recovery owner. Detailed renderer diagnostics are restricted and must remain
free of routes, credentials and protected content.

Permitted analytics include latency, result class, expiry, duplicate,
reconnect, incompatibility and failure rates. Analytics cannot issue cues,
alter registry state or become an authoritative checkpoint source.

## Conformance evidence required

Evidence must show valid applied and every negative state, identical and
conflicting duplicates, stale generation, wrong session, late response,
gateway restart, surface restart and uncertain reconciliation. Tests must prove
that applied is never interpreted as business completion and that diagnostic
paths disclose no route or security material.

## Current limitation

This is an accepted logical specification. The canonical schema, precise
renderer diagnostic vocabulary, persistence and transport acknowledgement
mapping are pending.

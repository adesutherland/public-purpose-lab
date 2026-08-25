# C-003: Command outcome and failure

Status: Working baseline

Version: 1.0.0

Owner: Every command receiver for its result; `INT-01` for delivery outcomes

## Purpose

`C-003` gives a caller a safe, correlated result for a command without
inventing a domain event. It distinguishes accepted, refused, expired,
duplicate and failed work and names who owns any recovery.

## Outcome concepts

Every outcome identifies itself and its command, exact contract version,
status, stable reason code, safe summary, completion time, retryability and
evidence references. A duplicate names the original outcome. A recoverable
failure may name a component or operator role responsible for the next action.

| Status      | Meaning                                                                                   |
| ----------- | ----------------------------------------------------------------------------------------- |
| `accepted`  | The owning boundary durably accepted the operation under the stated semantics             |
| `refused`   | Policy, authority, target, classification, compatibility or validation denied the request |
| `expired`   | The request was outside its permitted time window and no operation was applied            |
| `duplicate` | The same idempotency operation was already decided; no second operation was applied       |
| `failed`    | Processing could not safely complete; retry and recovery are explicit                     |

Acceptance does not claim that a later workflow, report or external action
completed. Business completion is reported by the owning domain contract.

## Repetition and recovery

- Duplicate delivery of the same semantic command returns or references the
  original decision without applying it again.
- Reusing the idempotency key for different semantic content is refused as an
  idempotency conflict, not treated as the prior command.
- A lost acknowledgement is reconciled against the same operation.
- A failure is retryable only when the receiver states that retry cannot
  duplicate a possibly accepted effect.
- Recovery never overwrites the original outcome or fabricates an accepted
  event.

## Disclosure and evidence

Reason codes are stable and safe for callers and operations. Summaries exclude
payload, credentials, secret material and unnecessary identity detail.
Evidence is linked through `C-004`, not embedded in the outcome.

Canonical source:
[JSON Schema](../../../../../contracts/common/c-003-command-outcome-and-failure.schema.json).
Conformance examples are listed in the
[fixture manifest](../../../../../contracts/common/fixtures.json).

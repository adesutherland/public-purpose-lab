# A-002 — Staged source release

Status: Working draft implemented for the Gate C validation and staging slice

Version: `0.1.0`

Owners: `CNT-01` source governance and `AUT-01` policy decision; `UX-02`
Workbench adapter

## Purpose

`A-002` makes the boundary between quarantined material and staged material
explicit. `CNT-01` records deterministic validation results for an immutable
source version. A named synthetic reviewer may then request release to staging;
`AUT-01` evaluates that protected action and `CNT-01` owns the final lifecycle
decision.

Validation does not establish that a source is true, authoritative, safe for
all later uses or free from malware. Staging does not approve a finding or
report and does not grant legal, professional or regulatory authority.

## Validation and lifecycle status

After successful `A-001` quarantine, `CNT-01` performs the bounded DS-03 checks:

1. admitted text or Markdown media type;
2. content remains present;
3. the retained body matches its recorded SHA-256 digest;
4. disallowed control characters are absent; and
5. a small, disclosed set of prompt-injection and protected-key markers is
   absent.

The resulting `source-lifecycle.status` identifies every check as passed or
failed and retains a safe reason code. Source content is not returned. A failed
validation leaves the immutable version quarantined and makes staging
ineligible.

These checks are deliberately deterministic assurance controls for the first
synthetic text path. They are not a general malware scanner, content moderation
service, truth assessment or evidence-quality judgement.

## Release command and outcome

`staged-source-release.command` identifies the environment, demonstration
session, engagement, immutable source version, authenticated synthetic
reviewer, application-session authority reference, purpose, correlation,
causation and idempotency key. The Presentation Gateway derives protected
fields from the established application session; the browser cannot assert
them.

`CNT-01` requires the version to exist in the same environment and session and
to have a successful validation result. It requests an `AZ-001` decision from
the separately deployed `AUT-01` workload. Only a permit for the exact actor,
role, purpose, action and source-version resource allows `CNT-01` to record
`staged` and emit `source.staged`.

A deny, not-applicable or indeterminate policy result fails closed and emits
`source.staging-refused` with a safe reason. An exact retry returns the retained
outcome; changed semantic input under the same idempotency key is refused.

## Ownership and privacy

- `CNT-01` alone reads the source body and owns validation and lifecycle state.
- `AUT-01` receives identifiers and bounded assertions, not source content.
- The Workbench and Operations views receive validation metadata, the named
  actor and a policy decision reference, not source content or credentials.
- `KNO-01` is not called by this slice. A staged fact becomes eligible input for
  the later processing slice but is not evidence that processing occurred.

## Failure behaviour

Missing, cross-environment, cross-session, unvalidated or previously refused
versions cannot be staged. Policy unavailability is an indeterminate refusal,
not an implicit permit. Database or event-publication failure preserves the
last conclusive component-owned state and uses the existing transactional
outbox recovery path.

## Evidence

- canonical schema and fixtures: `contracts/source/`;
- source-governance tests: `backend/components/cnt-01/src/lib.rs`;
- policy decision tests: `backend/components/aut-01/src/lib.rs`; and
- end-to-end system check: `tools/smoke-m3-native.sh`.

This working contract will be included in the complete Gate C candidate
published as final subject to review sign-off by exception. Until that
candidate is exercised and published, it remains an in-development baseline.
Acceptance will qualify only the bounded synthetic, single-instance Gate C
validation and staging transaction.

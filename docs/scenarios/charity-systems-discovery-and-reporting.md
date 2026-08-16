# Charity systems discovery and reporting

Status: founding demonstrator

## Problem

A mental-health charity has a patchwork of CRM, case-management, spreadsheet,
and manual reporting processes. Staff spend scarce time discovering where
information lives, reconciling inconsistent records, and assembling reports.

## Demonstration

Using an entirely synthetic organisation and data, show how interview notes,
system inventories, and sample exports can become a reviewable system map; how
replaceable adapters can ingest and reconcile records; and how an
evidence-linked report can be produced under explicit privacy and disclosure
rules.

## Questions to answer

- Can assisted analysis accelerate discovery without inventing certainty?
- Does the system expose gaps, conflicts, and unknowns rather than hide them?
- Can a reviewer trace every reported result to its source, transformation, and
  applicable rule?
- Can people correct the model and retain accountable control?
- Is the approach affordable and operable for a small organisation?

## Minimum end-to-end slice

1. Load one versioned synthetic source through a replaceable adapter.
2. Validate and map it without discarding conflicts or provenance.
3. Apply one explicit privacy, disclosure, or transformation decision.
4. Emit a correlated domain event and update independently owned state.
5. Require one meaningful human review or approval.
6. Produce one report element with an inspectable evidence chain.
7. Replay the scenario from a deterministic reset.

## Adversarial cases

- contradictory values from two sources;
- missing provenance or an unrecognised schema version;
- malicious content or prompt injection in a source field;
- an unauthorised disclosure request;
- an AI-generated claim with insufficient evidence;
- duplicate and out-of-order events; and
- a component failure after partial progress.

## Explicit exclusions

No real charity, donor, staff, client, or service-user data. The demonstration
does not claim to replace a CRM or case-management system and does not establish
legal compliance or production readiness.

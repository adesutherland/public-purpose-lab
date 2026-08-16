# Terms of reference

Status: founding working agreement
Last reviewed: 16 August 2026

## Purpose

Public Purpose Lab exists to research, build, demonstrate, and openly document
practical approaches to fragmented systems and operational workflows in
public-purpose organisations.

## Objectives

- Establish a secure, cloud-native reference platform for executable
  demonstrators.
- Develop reusable integration, eventing, identity, policy, observability, and
  assurance components.
- Explore how AI can reduce discovery and administrative effort without
  obscuring responsibility or weakening professional judgement.
- Demonstrate alternatives to expensive, monolithic enterprise solutions where
  a smaller composable approach may be appropriate.
- Publish useful patterns, failures, decisions, and evidence so that others can
  learn or contribute.

## Initial scope

- Charity and voluntary-sector system discovery, integration, and reporting.
- Operational UK health and social-care scenarios using synthetic data and
  simulated interfaces.
- Reusable, vendor-neutral component and event boundaries.
- Human-in-the-loop workflow and explicit decision authority.
- Privacy, security, provenance, and assurance mechanisms exercised by the
  demonstrators.
- Repeatable presentation scenarios that can be paused, inspected, replayed,
  and tested.

## Initially out of scope

- Clinical diagnosis, treatment recommendations, or autonomous clinical
  decision-making.
- Processing real patient, service-user, employee, or donor information.
- Claims of NHS endorsement, regulatory approval, legal compliance, production
  readiness, or guaranteed AI correctness.
- Replacing an organisation's entire CRM, EPR, case-management, or enterprise
  platform.
- Publishing third-party confidential information or material obtained under
  an NDA.
- Optimising for scale before a demonstrator establishes that the service idea
  is useful.

## Participation

Adrian Sutherland and Stephen Boyle are the founding participants. Specialists,
organisations, and contributors may be invited when they bring relevant domain
knowledge, technical scrutiny, access to a testable problem, or independent
challenge. Participation does not imply endorsement by an employer, charity,
NHS body, public authority, or other institution.

## Working cadence

The founding group normally meets on Thursdays. A lightweight session record
should capture the question being addressed, decisions, evidence reviewed,
actions, owners, and unresolved risks. Repository issues and architecture
decision records provide continuity between sessions.

## Decisions and stop authority

Founding decisions should be made by consensus wherever practical. Material
architecture, privacy, licensing, and scope decisions are recorded in the
repository. When consensus is unavailable, defer the irreversible choice or run
a bounded experiment that produces evidence.

Either founder may stop publication or demonstration on identifying a credible
confidentiality, privacy, security, legal, or safety concern. Record the concern
and its resolution without exposing protected information.

## Open-source and contribution position

The project is intended to be open source. A code licence, documentation
licence, contributor model, and intellectual-property position must be agreed
before accepting substantive external contributions. Until then, no
confidential or encumbered code, data, prompts, documents, or interface
specifications may enter the repository.

## Technology position

Rust, Kubernetes-compatible deployment, a modern web frontend, explicit events,
zero-trust boundaries, and externalised policy form the intended foundation.
cREXX may be used where it is the appropriate tool, including for rules,
transformations, scenario assets, automation, or integration logic. Its use is
not mandatory and should be justified by the same maintainability, security,
interoperability, and operational criteria as any other technology choice.

## Review

Review these terms after the first demonstrator reaches an end-to-end state, or
earlier if participation, data access, funding, publication, or external
commitments materially change.

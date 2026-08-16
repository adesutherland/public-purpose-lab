# ADR-0001: Grow architecture from end-to-end scenarios

Status: Accepted
Date: 2026-08-16

## Context

The Lab needs credible cloud-native foundations but does not yet have enough
evidence to define a large estate of independently deployed services. Empty
service scaffolding would create operational cost and imply boundaries that the
business scenarios have not proved.

## Decision

Build the smallest end-to-end path for the charity discovery and reporting
demonstrator. Represent domain and trust boundaries explicitly in code and
contracts, but deploy them separately only when ownership, security, scaling,
resilience, or independent change justifies it.

Use the care disruption and rebooking demonstrator to test which capabilities
are truly reusable before promoting them into shared platform components.

## Consequences

- Cloud-native operation remains a design target, not a reason to multiply
  services.
- Component contracts and ownership must be clear even when components share a
  process during an early experiment.
- Kubernetes packaging follows the first defined walking skeleton.
- Shared components require evidence from more than one scenario.

## Validation and review

Review after the first demonstrator completes a command-to-evidence path and
again when the second scenario exercises it. Reconsider a deployment boundary
when measured ownership, trust, scaling, failure containment, or release needs
cannot be met within the current shape.

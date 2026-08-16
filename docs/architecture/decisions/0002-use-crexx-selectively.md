# ADR-0002: Use cREXX selectively

Status: Accepted
Date: 2026-08-16

## Context

The founders have access to cREXX technology and expertise that may be valuable
for portable rules, transformations, automation, scenario assets, and
integration behaviour. Mandating it for every script or rule would couple the
Lab's architecture to a technology before individual needs and operational
trade-offs are understood.

## Decision

cREXX is an available and encouraged option where it provides a demonstrable
fit. It is not mandatory. Each material use must document its purpose, boundary,
permissions, resource limits, versioning, tests, observability, deployment, and
the simpler alternatives considered.

Rust remains the intended language for backend services. Declarative formats
such as JSON, YAML, TOML, schemas, and Kubernetes manifests remain appropriate
for configuration and contracts.

## Consequences

- The Lab can reuse and test cREXX assets without forcing unrelated components
  to adopt them.
- cREXX-backed behaviour must be operable and inspectable by the wider project.
- A scenario may choose another mechanism when it is simpler, safer, or better
  supported.
- No single technology choice substitutes for explicit policy ownership,
  sandboxing, testing, or audit evidence.

## Validation and review

Review after the first candidate cREXX asset is exercised end to end. Compare
clarity, safety, portability, testability, performance, deployment cost, and
operator experience with the most credible alternative.

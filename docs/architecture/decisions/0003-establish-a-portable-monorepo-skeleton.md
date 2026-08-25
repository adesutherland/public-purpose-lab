# ADR-0003: Establish a portable monorepo skeleton

Status: Accepted
Date: 2026-08-25

## Context

The Lab needs to move from conceptual architecture to executable evidence
without implying that the platform, its security controls or its logical
components already exist. The first repository shape must support development
on macOS, Linux and Windows, container-hosted demonstrations and later
Kubernetes deployment. It must also preserve logical ownership without turning
every component in the blueprint into a service.

Detailed event, workflow, storage, identity, observability, retrieval and cREXX
runtime choices have not yet been made.

## Decision

Use one repository with:

- a Cargo workspace for dependency-light Rust host and component packages;
- a pnpm workspace for TypeScript browser applications and shared UI packages;
- machine-readable catalogues linking all logical components and contract
  families to their documentation, maturity and source paths;
- separate Workbench, Director and Presentation browser builds over one shared
  UI boundary;
- OCI container definitions and a minimal Kubernetes base for the executable
  skeleton; and
- one CI path that checks catalogues, links, formatting, types, tests, builds,
  Kubernetes rendering and container construction.

Create source packages only when an executable slice needs them. The initial
Rust host and `IAM-01` package expose build and ownership boundaries but no
identity, certificate, grant or session behaviour. The browser surfaces expose
intended presentation boundaries but no live platform actions.

Exclude cREXX from the first build while retaining `RUL-01` as its potential
future integration boundary. Do not select an event broker, workflow engine,
database, retrieval store, identity provider, certificate mechanism, key store
or observability stack through this repository decision.

## Consequences

- Native builds give contributors a fast macOS, Linux or Windows development
  path; containers give demonstrations a common hosted form.
- Logical components may share a deployable until evidence requires a service
  boundary.
- A compilable package is labelled `repository-skeleton` and is not evidence of
  operational, security, compliance or production capability.
- Product and protocol choices remain explicit later ADRs rather than hidden
  build defaults.
- Adding a component requires synchronized catalogue, documentation, contract,
  test and deployment evidence.
- The first CI container builds provide hosted packaging evidence; local
  environments without an OCI runtime cannot provide that qualification.

## Validation and review

The decision is initially validated when:

1. the Rust workspace builds and its tests and lints pass;
2. all three browser surfaces type-check, test, build and render;
3. the catalogue accounts for every logical component and contract family;
4. the Kubernetes base renders without adding undeclared platform services;
5. CI constructs both container images; and
6. a clean checkout can reproduce the checks from the documented prerequisites.

Review the repository shape after the first contract-backed end-to-end slice,
before adding the cREXX runtime, and whenever measured trust, scaling, release
or failure-containment evidence calls for a new deployable boundary.

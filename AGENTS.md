# Repository guidance

Public Purpose Lab is an early-stage, open-source-intended lab for trustworthy
service integration and automation in charities and UK public services.

## Before changing the repository

- Read `VISION.md`, `PRINCIPLES.md`, `TERMS-OF-REFERENCE.md`, and the relevant
  scenario or architecture document.
- Do not imply NHS, government, charity, employer, clinical, regulatory, or
  production endorsement.
- Use synthetic data only unless the founders have recorded separate authority
  and governance.
- Record material architecture, privacy, security, licensing, and scope choices
  as architecture decision records.
- Keep public-purpose outcomes and accountable human authority visible.

## Engineering direction

- Prefer Rust for backend components and a modern TypeScript frontend.
- Design for Kubernetes-compatible operation without creating services that no
  demonstrated scenario needs.
- Use explicit commands, events, interfaces, ownership, correlation,
  idempotency, and versioned contracts.
- Treat privacy, identity, policy, audit, observability, and failure behaviour as
  architecture, not later additions.
- Use cREXX assets where they provide a clear fit for rules, transformations,
  scenario automation, or integration logic. Do not mandate cREXX or introduce
  it without documenting the value, trust boundary, and operational impact.

## Working rules

- Present a numbered plan before material architecture or scope changes and
  pause for founder approval of irreversible choices.
- Prefer the smallest end-to-end experiment that can produce decisive evidence.
- Keep documentation synchronized with implementation.
- Add focused tests for component contracts and end-to-end evidence for the
  scenarios they support.
- Never commit secrets, personal data, confidential material, or unlicensed
  third-party assets.

# Repository guidance

Public Purpose Lab is an early-stage, open-source-intended lab for trustworthy
service integration and automation in charities and UK public services.

At the start of a new session, read this file, `README.md`, `VISION.md`,
`PRINCIPLES.md`, `TERMS-OF-REFERENCE.md` and the relevant scenario or
architecture document. This public repository is authoritative for governance,
scenarios and architecture direction; the private `site-publicpurposelab-org`
repository owns website presentation and deployment.

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
- Treat cREXX as the preferred open implementation surface for inspectable
  business rules, transformations and scenario scripting where those
  responsibilities exist. It is not the mandated language for user interfaces,
  general services, storage or infrastructure. Document the value, trust
  boundary and operational impact of every integration, and document an
  exception when a different rules/scripting surface is selected.
- Map demonstrator components to the logical system blueprint maintained by
  Architecture Portal, and return implementation evidence and lessons to that
  blueprint. The private cross-portfolio direction is in
  `../site-architectureportal-org/docs/portfolio-content-direction.md`.

## Working rules

- Present a numbered plan before material architecture or scope changes and
  pause for founder approval of irreversible choices.
- Prefer the smallest end-to-end experiment that can produce decisive evidence.
- Keep documentation synchronized with implementation.
- Add focused tests for component contracts and end-to-end evidence for the
  scenarios they support.
- Never commit secrets, personal data, confidential material, or unlicensed
  third-party assets.

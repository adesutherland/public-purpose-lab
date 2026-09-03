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
- Do not close an implementation gate without a maintained Markdown progress
  and show-and-tell report plus a visually verified PDF distribution copy. The
  report must use real screenshots from a walkthrough of the exact evidenced
  build, connect them to the approved flow and acceptance steps, list the
  screens, functions and rules delivered, explain inherited context and
  limitations, and identify the next gate. Retain only synthetic,
  privacy-safe screenshots and record source revision and environment profile.
- Use review by exception for gate publication. Build, test, walk through,
  render and visually verify one complete candidate, then publish its Markdown
  and PDF as `Final — subject to review sign-off by exception` with the
  implementation. A successful founder review requires no status-only commit,
  repeat test run or regenerated report. If review identifies an exception,
  correct the affected material, rerun checks and walkthrough evidence in
  proportion to the change, and publish a new head that supersedes the earlier
  report. Do not repeat unaffected evidence merely to record approval.
- Never commit secrets, personal data, confidential material, or unlicensed
  third-party assets.

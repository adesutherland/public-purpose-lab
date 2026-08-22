<p align="center">
  <img src="docs/brand/public-purpose-lab-logo.svg" width="560" alt="Public Purpose Lab">
</p>

<p align="center"><strong>Open systems. Public purpose.</strong></p>

<p align="center"><a href="https://publicpurposelab.org">publicpurposelab.org</a></p>

Public Purpose Lab is an emerging open-source lab for trustworthy service
integration and automation. We explore how small, experienced teams can help
charities and public services understand fragmented systems, connect operational
workflows, and test affordable alternatives to opaque or monolithic platforms.

The Lab is in its founding phase. It is a place for disciplined experiments,
reusable components, and inspectable evidence—not a finished product, an NHS
programme, or a claim of production or regulatory readiness.

## What we are starting with

Two synthetic demonstrators will shape the shared foundations:

1. **Charity systems discovery and reporting** — map fragmented information,
   connect replaceable sources, expose conflicts, and produce evidence-linked
   reporting under explicit privacy rules.
2. **Care disruption and rebooking** — coordinate a simulated service
   disruption across appointments, capacity, priorities, approvals, and
   communications without hiding decisions in a central demo controller.

Both demonstrators use synthetic data and simulated external interfaces. They
must make uncertainty, human authority, policy decisions, and audit evidence
visible.

## Engineering direction

The intended technology foundation is cloud native and component oriented:

- Kubernetes-compatible deployment and operations;
- Rust for safety-sensitive backend services and shared components;
- a modern TypeScript web frontend with an accessible design system;
- explicit APIs, commands, domain events, correlation, idempotency, and
  versioned contracts;
- zero-trust boundaries, workload identity, least privilege, and observable
  policy decisions;
- privacy rules and other changeable policy kept outside service code where
  practical;
- replaceable adapters for charity, NHS, and UK social-care interfaces, added
  only when a demonstrator defines a real need; and
- cREXX as the preferred open implementation surface for inspectable business
  rules, transformations and scenario scripting where those responsibilities
  exist. It is not mandated for user interfaces, general services, storage or
  infrastructure, and every use must document its value and operational
  impact.

We will build the smallest complete path that can answer a meaningful question.
The architecture will grow from demonstrated needs rather than from an empty
estate of services.

## Portfolio relationship

[Architecture Portal](https://architectureportal.org) owns the shared
architecture method and logical system blueprint. Public Purpose Lab may apply
selected blueprint components in synthetic demonstrations and return specific
test results, operating observations, limitations and lessons. No integrated
portfolio demonstrator has yet been published.

## Repository guide

- [Vision](VISION.md)
- [Founding principles](PRINCIPLES.md)
- [Terms of reference](TERMS-OF-REFERENCE.md)
- [Governance](GOVERNANCE.md)
- [Initial roadmap](ROADMAP.md)
- [Architecture](docs/architecture/README.md)
- [Demonstrator briefs](docs/scenarios/README.md)
- [Brand foundation](docs/brand/README.md)
- [Contributing](CONTRIBUTING.md)
- [Security](SECURITY.md)

New contributors and agent sessions should also read `AGENTS.md` before making
changes. The website source and release configuration are maintained separately
in the private `site-publicpurposelab-org` repository; this public repository is
authoritative for the Lab's governance, scenarios and architecture direction.

## Project status and licensing

Adrian Sutherland and Stephen Boyle are the founding participants. The project
name and initial framing are approved; the repository structure and documents
remain open to refinement as the first demonstrator is designed.

The intended code and contribution licence has not yet been selected. Until it
is, this public repository is available for review, but external code
contributions are not being accepted. See [Governance](GOVERNANCE.md).

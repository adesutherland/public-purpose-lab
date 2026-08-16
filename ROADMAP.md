# Initial roadmap

The roadmap is organised around evidence, not dates or component counts. The
first ninety-day sequence is a working proposal.

## 1. Establish

- Confirm founding terms, governance, licence direction, and contribution model.
- Define the development environment and repository conventions.
- Record architecture principles, a threat model, and the synthetic-data rule.
- Build a thin Kubernetes-compatible walking skeleton only after its first
  end-to-end behaviour is defined.

## 2. Prove one path

Deliver the smallest charity reporting path with:

- one synthetic source and one replaceable adapter;
- one explicit policy or transformation;
- one domain event chain with correlation and idempotency;
- one accountable human checkpoint;
- one evidence-linked output; and
- one presenter-controlled scenario.

## 3. Challenge it

Introduce a conflicting source, failed component, malformed or malicious input,
unauthorised request, and AI uncertainty. Demonstrate how the system detects,
contains, records, and communicates each condition.

## 4. Generalise carefully

Use the care disruption and rebooking scenario to test which capabilities are
genuinely reusable. Extract shared components only when both scenarios provide
evidence for the boundary.

## Founding decisions still required

- Code, documentation, and contribution licences.
- Initial event transport, identity, policy, and observability choices.
- Local development and Kubernetes distribution.
- First synthetic source and report outcome.
- Criteria for selecting cREXX or another mechanism for an individual rule,
  transformation, or automation asset.
- Domain and operational reviewers for the two scenarios.

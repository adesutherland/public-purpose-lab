# C-002: Authority and purpose context

Status: Working baseline

Version: 1.0.0

Owner: `IAM-01` for validated context; every receiving component for its action
decision

## Purpose

`C-002` represents who or what requests an action, any independently
attributable initiating actor, the bounded authority asserted, the permitted
purpose and the constraints a receiver must evaluate. It preserves identity
types rather than compressing them into one user string.

M1 validates the structure and internal consistency of this context in a local
assurance profile. A reference such as `authenticationContextRef` is not proof
of authentication. M2 supplies the accepted identity bindings.

## Principal model

`requester` is the principal invoking the receiving boundary. `actor` is
optional and identifies a human or synthetic human on whose attributable
request a workload acts. The two are never substituted.

Every principal reference contains:

- type: external human, synthetic human, workload, operator or service owner;
- immutable, non-display principal identifier;
- environment and issuer; and
- an optional synthetic or workload trust-domain reference.

The context also contains bounded roles, delegated contract authorities,
purpose, constraints, policy version and an optional safe decision reference.
Constraints may restrict engagement, Demonstration Session, information class,
target or time.

## Authority rules

- Authentication and role mapping do not themselves authorise a domain action.
- The receiver evaluates the requested action against its current policy and
  records the decision version.
- An intermediary may reduce authority or constraints but cannot add a role,
  purpose, target or information class.
- A workload acting for a person carries both principals. A workload cannot
  invent, replace or silently drop the actor.
- A synthetic principal is valid only in its environment and synthetic trust
  domain and can never acquire external-human or production authority.
- Operator and platform access do not imply business approval or report-release
  authority.
- Missing, excessive, expired, conflicting or wrong-environment authority is
  refused through `C-003`.

## Privacy, retention and evidence

The context carries identifiers and safe references, not credentials, raw
identity assertions or provider tokens. Audit may retain principal type and an
irreversible identifier digest with policy, purpose, correlation and outcome.
Enumeration-sensitive identity detail remains behind an authorised view.

Canonical source:
[JSON Schema](../../../../../contracts/common/c-002-authority-and-purpose-context.schema.json).
Conformance examples are listed in the
[fixture manifest](../../../../../contracts/common/fixtures.json).

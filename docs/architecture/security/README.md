# Security architecture

Security is a framework responsibility rather than an identity-component
appendix. These documents define the accepted framework baseline and the
narrower M1 threat baseline against which component contracts and
implementation evidence are assessed.

The baseline distinguishes a visible `local-synthetic` trust profile for
isolated scratch environments from the `managed` trust required for hosted,
shared, production-like, production or non-synthetic-data environments.

- [Framework security model](framework-security-model.md) — trust zones,
  principals, authority, information classes, recovery domains and enduring
  invariants.
- [M1 threat model](m1-threat-model.md) — threats and controls exercised by the
  common interaction and reference-runtime slice.
- [M2 identity and synthetic-access threat model](m2-threat-model.md) — threats,
  controls and explicit limits for the local-synthetic identity reference path.

Both documents are expected to evolve as demonstrators expose better evidence.
A revision is controlled and attributable; it does not silently weaken an
accepted invariant or make an implementation claim broader than its evidence.

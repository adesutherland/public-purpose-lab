# Security architecture

Security is a framework responsibility rather than an identity-component
appendix. These documents define the accepted framework baseline, bounded
milestone baselines and working drafts against which component contracts and
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
- [M3 Scenario Director threat model](m3-threat-model.md) — accepted logical
  threats, required controls and implementation gates for `CTL-01` and
  `D-001` to `D-004`; no physical binding is accepted by implication.
- [M3.2 presentation and hosted-binding threat extension](m3-2-presentation-threat-extension.md)
  — accepted binding threats and evidence gates for `CTL-02`, browser
  delivery, presenter identity, event transport and the first Google Cloud
  binding.
- [M3.3 runtime-binding threat extension](m3-3-runtime-binding-threat-extension.md)
  — accepted threats and evidence gates for canonical packages, component
  state, bounded assurance controls and the first executable composition.
- [M3.4 identity and resilience threat extension](m3-4-identity-and-resilience-threat-extension.md)
- [Gate C source-intake threat extension](m4-source-intake-threat-extension.md)
  — implemented development controls and remaining hosted gates for external
  identity, application sessions, synthetic sign-in, managed signing and
  restart behaviour.

These documents are expected to evolve as demonstrators expose better
evidence. A revision is controlled and attributable; it does not silently
weaken an accepted invariant or make an implementation claim broader than its
evidence. A working draft becomes a milestone baseline only through founder
review.

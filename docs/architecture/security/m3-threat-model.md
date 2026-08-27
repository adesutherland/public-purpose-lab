# M3 threat model: Scenario Director control contracts

Status: Accepted M3.1 logical threat baseline; implementation bindings pending

Version: 0.1.0

Last reviewed: 27 August 2026

Governing decision:
[ADR-0011](../decisions/0011-establish-the-m3-scenario-control-invariants.md)

## Scope and claim boundary

This threat model covers the proposed `CTL-01` Scenario Director boundary and
the `D-001` to `D-004` logical contracts. It covers scenario-package admission,
Demonstration Session lifecycle, reset, logical time, bounded fault control,
readiness, checkpoints and their interaction with the accepted M1 common
contracts and M2 identity/authorisation principles.

The recommended assurance scenario is one Director Console, a Workbench shell
and a Presentation Surface, named environment-scoped synthetic actors, semantic
presentation intent, lifecycle controls, checkpoints and safe adverse cases.
It uses synthetic data and approved public material only.

M3.1 selects no runtime, database, event broker, API, browser protocol,
presenter identity provider, surface-binding mechanism or package-signing
profile. `CTL-02` and `P-001` to `P-004` are specified in M3.2 and will extend
this analysis for surface impersonation and cue delivery. This draft therefore
defines required security properties, not an implemented or production-
qualified control.

It is not evidence of legal, regulatory, clinical, professional or production
compliance.

## Protected assets

- integrity and provenance of admitted scenario packages;
- presenter, Director workload, synthetic actor and component attribution;
- authoritative Demonstration Session identity, lifecycle state and revision;
- separation of scenario coordination, presentation and business authority;
- idempotency, expiry, ordering and restart reconciliation;
- containment of reset, logical-time and fault operations;
- integrity and freshness of readiness and checkpoint evaluations;
- confidentiality of credentials, grants, sessions, routes and protected
  security state;
- component-owned business records, substantive evidence and audit history;
- environment and synthetic trust-domain isolation; and
- accurate maturity, readiness and assurance claims.

## Trust boundaries and assumptions

| Boundary                     | Trust position                                                   | Required property                                                                                                                |
| ---------------------------- | ---------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| Director Console and browser | Untrusted presentation client until authenticated and authorised | No direct state, event-broker, key or private-store access; backend enforcement for every protected action.                      |
| `CTL-01` control state       | Authoritative for scenario coordination only                     | Durable revisions, least privilege, safe recovery and no business or identity authority.                                         |
| Package and fixtures         | Untrusted content until admitted                                 | Immutable identity/digest, provenance, synthetic classification, bounded resources and no routes, secrets or executable content. |
| Component command boundary   | Owned by each receiving component                                | Current identity, authority, purpose, contract and domain-state validation; Director cannot bypass refusal.                      |
| `IAM-01` / `AUT-01`          | Accepted M2 logical identity and policy boundaries               | Principal separation, environment binding, fail-closed decisions and obligations.                                                |
| `CTL-02` / presentation      | Deferred M3.2 binding                                            | Authenticated surface registration and semantic cues with no business authority or routes.                                       |
| Reset/time/fault adapter     | Elevated test capability inside one owner boundary               | Explicit allow-list, closed parameters, containment, expiry, evidence and independent authority.                                 |
| Observation/evidence source  | Authoritative only for its owned claim                           | Version, freshness, scope, classification and access checks; no inference from appearance.                                       |
| Platform/operator            | Administrative capability, not presenter or business authority   | Named support actions, least privilege and no ambient impersonation.                                                             |

A local process, container, Kubernetes namespace, service account, event-channel
subscription or network location is not proof of identity or authority.
Co-deployment of `CTL-01` and `CTL-02` does not merge their logical powers.

## Security objectives

M3 control must:

1. admit only attributable, immutable, synthetic-scope packages with bounded
   declarative content;
2. require authenticated presenter and workload authority for every protected
   control action;
3. ensure business commands are separately authorised and enforced by their
   owners;
4. ensure a presentation cue can change presentation only;
5. prevent duplicate, delayed, conflicting or restarted work from duplicating
   accepted effects;
6. prevent logical scenario time from influencing protected security time;
7. constrain reset and faults to owner-published test capabilities;
8. preserve evidence and security-state continuity across reset and recovery;
9. derive readiness and checkpoints only from current attributable sources;
10. prevent one session, surface or environment from controlling another; and
11. expose safe failure and limitation evidence without credentials, routes or
    sensitive content.

## Threat register

The controls below are required by the proposed logical baseline. Their actual
strength must be demonstrated again for each selected implementation binding.

| ID        | Threat                                                                                            | Required M3.1 control                                                                                                                                            | Residual risk or later gate                                                     |
| --------- | ------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| `T-M3-01` | A caller forges the presenter, actor or Director workload                                         | Typed principals through `C-002`; authenticated M2 binding; both requester and actor retained; receiver-side enforcement                                         | Presenter and workload authentication bindings require M3.2 ADRs and evidence   |
| `T-M3-02` | The Director uses its workload identity as human or business authority                            | `CTL-01` least-privilege contract authority; no principal substitution; every business command independently authorised by its owner                             | Component implementations must prove consistent enforcement                     |
| `T-M3-03` | A package is altered, substituted or ambiguously interpreted                                      | Immutable package version and digest; exact compatibility; provenance and admission record; unsupported fields refused                                           | Canonicalisation, signing and distribution profile are unselected               |
| `T-M3-04` | A malicious package embeds a secret, route, endpoint or executable instruction                    | Declarative closed content; prohibited-content validation; no URL-based cue, shell, SQL or arbitrary code; synthetic-only profile                                | Content and archive parsing limits require implementation-specific tests        |
| `T-M3-05` | Package admission is mistaken for permission to perform every action                              | Admission remains descriptive; current identity, policy, purpose and component state revalidated for every command                                               | User-interface wording and operator training remain important                   |
| `T-M3-06` | A presentation cue performs a hidden business mutation                                            | Cues use a separate presentation contract and authority; presentation surfaces cannot receive domain-command authority; business checkpoints require owner facts | `P-003` schema and browser binding are M3.2 gates                               |
| `T-M3-07` | Command acceptance, a cue outcome or visible page is reported as business completion              | `C-003` acceptance separated from facts; `D-004` claim classes; no URL/DOM inference; source ownership and evidence required                                     | Detailed domain facts arrive in later milestones                                |
| `T-M3-08` | Duplicate or retried lifecycle commands advance state more than once                              | Required idempotency, semantic-content conflict check, expected revision and one durable state fact                                                              | State transaction and multi-replica binding are unselected                      |
| `T-M3-09` | Delayed or out-of-order work reopens or controls a stopped/reset run                              | Monotonic session revisions, terminal states, revalidation of current session and new successor session after reset                                              | Transport expiry and dead-letter behaviour require M3.2 ADR                     |
| `T-M3-10` | Director restart blindly repeats a possibly accepted business command, cue or fault               | Durable operation identity; classify incomplete work; reconcile with owner before retry; uncertainty fails closed                                                | Persistence and atomicity mechanism require implementation evidence             |
| `T-M3-11` | One session's command, grant, surface or fact is accepted in another                              | Environment, package, session, target and correlation binding; successful reset creates a new session; old work cannot cross successor boundary                  | `CTL-02` surface proof and each component binding need conformance tests        |
| `T-M3-12` | Scenario logical time extends a grant, certificate, session, message or policy decision           | Logical and operational time are separately labelled; security validation ignores logical time; unsupported separation is refused                                | Protected clock source, drift and tolerance remain profile-specific             |
| `T-M3-13` | Reset deletes evidence, races active work, loses security history or becomes a database wipe      | Stop-before-reset; component-owned allow-listed reset; state classes and retained evidence explicit; no arbitrary storage instruction; partial result visible    | Each reset adapter needs an ADR, least privilege and recovery tests             |
| `T-M3-14` | Reset reuses old grants or bindings in a nominally clean run                                      | Old session becomes `superseded`; successor has a new identifier; bindings and idempotency scope do not transfer                                                 | Dependent cleanup and revocation timing require M3.3/M3.4 evidence              |
| `T-M3-15` | Scenario reset silently repairs or clones a damaged trust domain                                  | Environment recovery remains outside `D-003`; failed continuity creates a new M2 trust domain                                                                    | Managed same-environment recovery remains out of M3 scope                       |
| `T-M3-16` | A fault profile escapes its target, persists invisibly or weakens security                        | Target-owned closed profile, separate authority, environment/session scope, maximum duration, explicit clear and observable state; no identity/audit weakening   | Concrete fault mechanisms and Kubernetes permissions need later review          |
| `T-M3-17` | An attacker uses reset/fault controls as an administrative interface                              | No arbitrary code, SQL, shell, route or network instruction; separate least-privilege authority; receiver-owned validation                                       | Operator and runtime access controls require binding evidence                   |
| `T-M3-18` | Stale, forged or unauthorised observations produce false readiness                                | Owner identity, contract/version, subject, session, revision, freshness, integrity and audience checks; unknown fails closed                                     | Observation authentication and delivery binding are unselected                  |
| `T-M3-19` | A late fact from a previous run satisfies the current checkpoint                                  | Package/session/correlation binding and source revision; successor session isolation; prior evaluations retained                                                 | Domain contract correlation must be implemented consistently                    |
| `T-M3-20` | Readiness or checkpoint rules execute unbounded or malicious code                                 | Initial rules are declarative, deterministic and bounded; unknown rule kinds refused; no executable package content                                              | Any later cREXX or other rule engine needs sandboxing, ADR and threat extension |
| `T-M3-21` | Logs, events, evidence or browser history expose credentials, grants, sessions or internal routes | Safe identifiers and `C-004` references only; explicit exclusions; semantic cues contain no routes; disclosure scanning                                          | Broker, frontend, crash and support-tool bindings need end-to-end tests         |
| `T-M3-22` | Oversized packages, high-rate controls or checkpoint fan-out exhaust resources                    | Declared resource limits, bounded counts/windows, admission validation, expiry and safe refusal                                                                  | Concrete size, rate and concurrency limits require performance evidence         |
| `T-M3-23` | Compromised `CTL-01` becomes a signing oracle or policy bypass                                    | Director can request but not sign grants; signer and `AUT-01` independently constrain requests; component owners enforce outcomes                                | Co-deployment and credential access need least-privilege review                 |
| `T-M3-24` | Operator access is used as presenter, synthetic actor or report authority                         | Principal separation, named recovery actions, restricted diagnostics and no ambient impersonation                                                                | Deployment RBAC and break-glass policy remain unselected                        |
| `T-M3-25` | M3 design or a successful demo is presented as production, real-data or compliance capability     | Working-draft and synthetic-only labels, explicit non-goals and evidence-based maturity                                                                          | Accurate publication and founder review remain governance controls              |

## Recommended assurance and misuse cases

The first executable evidence package should include:

- one authorised presenter operating one session through prepare, start, pause,
  resume, complete or stop and reset;
- one Workbench shell and one Presentation Surface role, registered later
  through M3.2 rather than browser URLs;
- two named synthetic human actors using separate, application-bound M2 grants;
- one separately authorised synthetic business command and authoritative fact;
- one semantic cue and presentation-only outcome;
- one business checkpoint and one presentation checkpoint proving that neither
  can satisfy the other;
- duplicate and changed-content lifecycle commands;
- delayed start/resume after stop and a prior-session fact after reset;
- Director restart before and after an accepted operation, including uncertain
  acknowledgement;
- a partial reset that prevents successor readiness;
- a logical-time change alongside an expired M2 grant, proving protected time
  is unaffected;
- one owner-published, bounded fault that expires or clears after the expected
  safe failure;
- an unauthorised presenter, mismatched workload, wrong environment and wrong
  session;
- a package containing a route, credential-like field, executable instruction,
  oversized resource declaration and unsupported rule;
- missing, stale, incompatible and unauthorised readiness observations; and
  checkpoint evaluation after restart; and
- disclosure scanning of events, outcomes, logs, browser history, evidence and
  support views for routes and usable security material.

## Failure and recovery requirements

- No error before a durable decision reports acceptance.
- An uncertain operation names its owner and blocks unsafe automatic progress.
- A corrupt or unavailable control-state record makes the affected session not
  ready; the runtime does not erase or silently repair history.
- Restart does not extend an operation, grant, cue, fault or observation
  validity window.
- Partial reset retains every conclusive target outcome and creates no ready
  successor.
- A new environment trust domain does not accept prior grants, sessions,
  bindings or authority even when package and actor display names match.
- Recovery evidence is privacy-minimised and cannot be used to replay a control
  or sign-in operation.

## Implementation-binding gates after M3.1

Before implementation, the following decisions require founder review and
ADRs where indicated:

1. package representation, schema, canonical digest, signature and
   distribution integrity;
2. presenter authentication and Director workload identity;
3. event/command transport, delivery, expiry, dead-letter and disclosure
   profile;
4. `CTL-02` surface capability, registration, binding, reconnect and cue
   protocol;
5. control-state persistence, transaction, idempotency and restart
   reconciliation;
6. logical-time mechanism and exact opted-in component contract;
7. reset and fault adapters, permissions, containment and recovery;
8. operational and audit evidence storage, retention and tamper evidence;
9. resource, rate, timeout and single-/multi-instance deployment profiles; and
10. local-container and Minikube enforcement differences; and
11. the ADR-0012 hosted-preview lifecycle, including operator authority,
    short-lived deployment identity, mandatory managed trust, automatic expiry,
    teardown reconciliation, residual endpoints/resources and cost-abuse
    controls.

Each binding extends this threat model and supplies positive, negative,
restart, disclosure and profile-specific evidence. Reusing an accepted JSON
shape or Kubernetes component does not inherit assurance automatically.

The disposable M3.2 Google Cloud infrastructure spike does not qualify a hosted
application or identity path. Before the M3.3 private application smoke, the
selected cloud services and deployment identity extend the trust-boundary and
threat register. Before any shared M3.4 demonstration, the managed root,
presenter/workload authentication, protected state and environment-lifecycle
controls require explicit negative and recovery evidence.

## Residual risk and review outcome

At M3.1 the largest residual risks are deliberately unresolved physical
bindings: presenter and surface authentication, package integrity, persistence
atomicity, event delivery, test-adapter privilege and browser/session
containment. No implementation should begin by choosing those products before
the logical boundaries and successor-session reset rule are reviewed.

The founders accepted this document as the M3.1 logical threat baseline on 27
August 2026. Acceptance does not promote `CTL-01` to implemented, demonstrated
or production-ready and does not accept any of the deferred implementation
bindings.

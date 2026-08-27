# ADR-0019: Bind M3 controlled time, reset and faults to bounded runtime adapters

Status: Accepted

Date: 2026-08-27

## Context

`D-003` makes repeatable reset, logical-time and adverse-case behaviour
explicit without granting the Scenario Director arbitrary administrative
power. M3.3 needs concrete mechanisms narrow enough to implement and test.

A replaceable operational clock is useful for code testing, but exposing it as
a scenario command would allow a demonstration to change security expiry.
Database truncation, shell scripts and Kubernetes fault tools would similarly
give the Director powers unrelated to its scenario authority.

The first walking skeleton owns only Director and presentation control state.
It has no business dataset that needs reset and no evidence for general chaos
engineering.

## Decision

Implement separate interfaces for protected operational time and declared
scenario logical time.

Operational time uses system UTC for attributable timestamps and monotonic time
for elapsed deadlines where applicable. It determines message, cue,
registration, application-session, policy, fault and idempotency validity. No
`D-003` command, scenario package or UI can change it. Unit and process tests
may inject a test operational clock only through a test-only construction path
that is unavailable in runtime profiles.

M3.3 scenario logical time uses manual-step progression:

- `SetScenarioLogicalTime` establishes the package-declared initial instant for
  one opted-in Demonstration Session;
- `AdvanceScenarioLogicalTime` moves it forwards within a closed package and
  runtime bound; and
- every change records a revision plus observed operational issue and
  acceptance times.

Backwards movement is refused. Pause does not stop operational time or silently
alter logical time. Any target unable to keep the two sources separate reports
the logical-time capability as not ready.

Implement two semantic reset adapters:

- `director-control-baseline`, owned by `CTL-01`; and
- `presentation-registry-baseline`, owned by `CTL-02`.

Each adapter accepts only its named operation and session scope. It terminates
or supersedes disposable control records needed for a clean successor while
retaining prior operation, security, idempotency and evidence history. Reset is
allowed only after the current session reaches an accepted terminal state and
creates a new successor session only after both required targets succeed
conclusively.

Reset cannot delete or repair trust roots, issuer state, grants, revocations,
application sessions, audit history, broker security, infrastructure or another
component's state.

Implement one M3.3 fault profile:
`presentation-cue-delay`. It is a one-shot `CTL-02`-owned delay of the next cue
for one environment, session, slot and operation. Parameters use a closed,
bounded duration schema; activation has a short operational-time expiry and
automatic/manual clear. It may demonstrate delayed or expired presentation but
cannot change identity, authorisation, audit, operational time, another session
or business state.

Fault controls are present only in local development-assurance and automated
test profiles during M3.3. The private hosted health/contract smoke asserts
that they are unavailable.

## Alternatives considered

- **Change the process or operating-system clock:** rejected because it affects
  security, logs and unrelated components and is not a scenario capability.
- **Use scenario time for all expiry in tests:** rejected because it would prove
  precisely the unsafe coupling the architecture prohibits.
- **Reset SQLite files or volumes:** rejected because it bypasses component
  ownership, erases evidence and cannot reconcile partial success.
- **Run arbitrary scripts or SQL:** rejected because a package or presenter
  would gain administrative and data-destruction authority.
- **Use Kubernetes/network chaos tooling immediately:** capable but broader than
  the first component-owned failure question and likely to expand operator and
  cluster permissions.
- **Implement several fault profiles:** deferred until the one-shot delay
  proves the contract, containment, expiry and evidence path.

## Consequences

- Tests can advance the business story while security and transport expiry
  continue under real operational time.
- Reset and fault behaviour remain explicit component contracts instead of
  hidden test scripts.
- The first reset does not recreate business fixtures or application sessions;
  those owners join later scenarios through their own adapters.
- Manual-step logical time is less theatrical than a continuously advancing
  simulation clock but is deterministic and sufficient for M3.3.
- Fault coverage is intentionally narrow; M3.4/M3.5 will add cases only where
  owner containment and value are demonstrated.

## Validation and review

Evidence must demonstrate:

- logical-time set/advance within bounds and refusal of backward, stale,
  unscoped or over-bound requests;
- cue, registration, message and session expiry remaining controlled by
  operational time during logical-time changes;
- stop-before-reset, duplicate reset, partial reset, uncertain target and
  successful successor-session behaviour;
- retention of prior evidence and refusal to alter identity/security state;
- one-shot delay activation, effect, automatic expiry, explicit clear,
  duplicate/conflict handling and cross-session refusal;
- no generic time, SQL, shell, file, network or Kubernetes control surface; and
- fault/test adapters absent from hosted/shared readiness and routes.

Review the bindings when a domain component needs logical time or reset, when a
continuous scenario clock is justified, or when an adverse case requires
platform-level fault injection.

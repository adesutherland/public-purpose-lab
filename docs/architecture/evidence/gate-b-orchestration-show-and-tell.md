# Gate B: environment, identity and portal orchestration

**Draft progress report, system-test record and show-and-tell**

| Evidence field        | Recorded value                                                                    |
| --------------------- | --------------------------------------------------------------------------------- |
| Status                | Implemented baseline; full visual walkthrough, PDF and founder acceptance pending |
| Automated walkthrough | 1 September 2026, local synthetic Minikube environment                            |
| Source revision       | Working tree on `temp/gate-b-orchestration`; exact review revision pending        |
| Local image           | `public-purpose-lab/m3-runtime:gate-b`                                            |
| Environment           | `environment-c5ef1307484826d1b598ad73a08d31b6`                                    |
| Cluster profile       | `public-purpose-lab-gate-a`                                                       |
| Trust profile         | Environment-local synthetic root; synthetic data only                             |
| Canonical requirement | `docs/scenarios/demonstration-scenarios-and-functional-requirements.md`           |

> This is partial in-development evidence. It does not close Gate B and does not establish production, compliance, legal, regulatory or clinical assurance.

## Outcome at a glance

The implemented Gate B baseline turns the inherited presentation-control
skeleton into the approved DS-01/DS-02 journey. The Director now shows the
environment and admitted scenario, creates and controls the Demonstration
Session, assigns a backend-only synthetic reviewer to Workbench, and requests
target-owned semantic views. Presentation and Workbench return conclusive view
outcomes, while Operations receives the correlated control, identity and view
events.

Source-intake controls are visible but deliberately non-submitting. Gate C
will add the first business process and component-owned source state.

## Acceptance position

| Requirement                                | Current evidence                                                        | Position                                    |
| ------------------------------------------ | ----------------------------------------------------------------------- | ------------------------------------------- |
| Environment, trust and presenter visible   | Live Director screen and authenticated environment endpoint             | Implemented; one screenshot retained        |
| Admitted scenario and limitations visible  | Live `DIR-CATALOGUE`                                                    | Implemented; one screenshot retained        |
| Session created without synthetic reviewer | Automated walkthrough checks the separate creation and assignment steps | Achieved automatically                      |
| Synthetic reviewer visible in Workbench    | Context API checks actor, role, environment, trust and expiry           | Achieved automatically; screenshot pending  |
| `PRES-INTRO` target-owned outcome          | SSE cue and `P-004 applied` outcome                                     | Achieved automatically; screenshot pending  |
| `WB-ENGAGEMENT` and `WB-SOURCE-INTAKE`     | Separate Workbench cues and outcomes                                    | Achieved automatically; screenshots pending |
| Ordinary Workbench navigation              | Implemented browser controls with no business event                     | Browser walkthrough pending                 |
| Unsupported view refused                   | Director returned HTTP 409 and emitted `view.refused`                   | Achieved automatically                      |
| Pause, stop, reset and session termination | State and successor checks passed                                       | Achieved automatically                      |
| Exact-build show-and-tell PDF              | Requires the remaining browser screenshots and visual QA                | Pending; gate remains open                  |

## Demonstrated flow

| Step | Actor and screen                                      | Component/event path                                            | Expected visible result                                        |
| ---- | ----------------------------------------------------- | --------------------------------------------------------------- | -------------------------------------------------------------- |
| 1    | Presenter signs into `DIR-ENVIRONMENT`                | Director validates external application session                 | Presenter, environment, runtime and trust profile appear       |
| 2    | Presenter opens `DIR-CATALOGUE`                       | Director evaluates the admitted package against the environment | Governed source assurance is ready with honest limitations     |
| 3    | Presenter creates and prepares the session            | `D-002` lifecycle plus bounded logical time                     | Session and revision become visible; no synthetic reviewer yet |
| 4    | Surface operators register Presentation and Workbench | `P-002` role-specific registrations                             | Each target owns a current connection generation               |
| 5    | Presenter assigns `synthetic-reviewer`                | Director → NATS → IAM-01 → target Workbench backend             | Workbench shows actor, reviewer role, trust profile and expiry |
| 6    | Presenter starts and shows introduction               | `P-003 pres-intro` → Presentation → `P-004 applied`             | Audience sees scenario purpose, actors, outcome and limits     |
| 7    | Presenter opens engagement and intake                 | `P-003 wb-engagement`, then `wb-source-intake`                  | Workbench changes view without a route or DOM instruction      |
| 8    | User navigates within Workbench                       | Local accessible navigation only                                | Same views are usable without Director; no business event      |
| 9    | Presenter tests unsupported view and pauses           | Pre-delivery refusal; `D-002 pause`                             | Refusal is explicit and `DIR-RUN` reports paused               |
| 10   | Presenter stops and resets                            | Synthetic bindings terminated; successor created                | Presenter remains signed in; old Workbench actor is terminated |

## Screenshot evidence retained so far

The exact-build visual evidence set is incomplete, so no Gate B PDF has yet
been produced.

![Figure 1. Authenticated Director environment and admitted scenario catalogue in the local Minikube Gate B build.](gate-b-orchestration/01-director-environment-and-catalogue.png)

The remaining closure set must capture the active `DIR-RUN`, `PRES-INTRO`,
Workbench reviewer banner, `WB-ENGAGEMENT`, `WB-SOURCE-INTAKE`, paused state and
the correlated Operations timeline from the exact review build.

## Automated system-test evidence

The latest complete CLI-driven walkthrough used Demonstration Session
`session:a02b5450-2fc0-4f15-801e-4be81729d1c3` and checked:

- CSRF refusal with HTTP 401;
- distinct audience and Workbench surface sessions;
- audience actor `synthetic-audience-user` and Workbench actor
  `synthetic-reviewer` with role `workbench-reviewer`;
- environment ID, environment-local synthetic trust and bounded expiry;
- `pres-intro`, `wb-engagement` and `wb-source-intake` cue delivery and applied
  target outcomes;
- HTTP 409 for `wb-not-admitted`;
- pause/resume, stop, reset and termination of both synthetic sessions; and
- a clean successor session with no inherited checkpoint or logical-time state.

The Operations query for an earlier complete Gate B session returned
`scenario.started`, `scenario.step.requested`, `scenario.paused`,
`scenario.stopped`, `scenario.reset`, `surface.registered`,
`synthetic-session.established`, `synthetic-session.terminated`,
`view.requested`, `view.applied` and `view.refused` under the scenario
correlation scope.

## Functions, rules and components changed

The detailed implementation baseline is recorded in
`docs/architecture/gate-b-orchestration.md`. In summary, Gate B changes CTL-01,
CTL-02, IAM-01 integration, the Director, Presentation, Workbench and
Operations surfaces, the scenario package and the presentation examples. It
does not add behaviour to DOM-01, CNT-01, KNO-01, WRK-01, RPT-01 or AUD-01.

## Limitations and next steps

- The Operations projection remains transient rather than durable audit
  evidence.
- Surface links are environment configuration; orchestration remains semantic
  event based.
- No source content is sent, stored or processed and no engagement record is
  created.
- The Gate B browser walkthrough and screenshot set must be completed on the
  exact review revision before rendering and visually checking the PDF.
- Gate C then implements DS-03: upload/paste, quarantine, validation, staging,
  processing status, logs and events as the first functional business path.

# Care disruption and rebooking

Status: second founding demonstrator

## Problem

A consultant or specialist becomes unavailable, affecting scheduled
appointments and creating a time-sensitive coordination problem across people,
services, capacity, priorities, and communication channels.

## Demonstration

Using synthetic people and appointments with simulated health and care
interfaces, show how a disruption is received; how affected work is identified;
how policy-constrained options are proposed; where human approval is required;
and how resulting actions, notifications, refusals, and unresolved cases are
tracked.

## Questions to answer

- Can independently owned components collaborate without a shared database or
  hidden central decision-maker?
- Are prioritisation, privacy, communication, and human-authority rules visible
  and testable?
- Can the scenario be replayed with changed capacity or policy and produce an
  explainable result?
- Does failure degrade safely without losing affected people or duplicating
  actions?

## Candidate event flow

1. A simulated service publishes a `ServiceAvailabilityChanged` fact.
2. The scheduling component identifies affected synthetic appointments.
3. Capacity and policy components return versioned, explainable constraints.
4. A coordination component proposes bounded options rather than selecting a
   clinical outcome.
5. A human participant approves, changes, defers, or rejects each consequential
   action.
6. Adapter components simulate booking changes and communications.
7. Evidence records connect the original disruption to every outcome.

Event names are illustrative until an accepted domain model and schemas exist.

## Adversarial cases

- stale or contradictory capacity;
- duplicate disruption events;
- an unauthorised user attempting reprioritisation;
- policy conflict or missing consent;
- unavailable communication channel;
- partial rebooking failure; and
- a case that cannot be resolved within the scenario's authority.

## Explicit exclusions

No diagnosis, treatment recommendation, autonomous clinical decision, real
patient data, or connection to a live NHS or social-care system. The
demonstrator tests operational coordination concepts only.

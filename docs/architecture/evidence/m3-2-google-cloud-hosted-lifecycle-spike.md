# M3.2 Google Cloud hosted-lifecycle spike

Status: Accepted implementation evidence for the infrastructure lifecycle slice

Evidence date: 27 August 2026

Related decisions: [ADR-0012](../decisions/0012-introduce-a-cost-controlled-google-cloud-hosted-preview-during-m3.md)
and [ADR-0016](../decisions/0016-bind-the-first-hosted-preview-to-gke-autopilot.md)

## Claim and boundary

One infrastructure-only Google Cloud preview was created through the protected
operator path, observed in a ready state and conclusively deactivated. This
closes the M3.2 disposable infrastructure spike, not the wider M3 milestone.

The exercise deployed no application, application ingress, external
application endpoint, database, persistent application data, real data or
shared synthetic sign-in. Managed root and issuer keys remained disabled. It
does not qualify a shared demonstration, production environment, compliance
control or transfer of legal or professional responsibility.

## Exercised profile

- Project: `public-purpose-lab` (`723170534380`)
- Region: `europe-west2`
- Activation: `m3-2-20260827-04`
- Immutable infrastructure revision:
  `d66d0187270f0ba1f2d0cbd0010e870003770705`
- Attributable operator: `github:adesutherland`
- Declared lifetime: one hour
- Actual mode: explicit manual `off` after the created inventory was captured

The activation definition contained three resources: a custom VPC, a regional
subnet and a GKE Autopilot cluster. The retained foundation contained separate
versioned state/evidence buckets, Artifact Registry, lifecycle identities,
short-lived GitHub federation, an expiry queue and a pinned Cloud Build
teardown trigger. Foundation and activation state remained separate.

## Result

| Evidence point      | Result                                                                                                                  |
| ------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| Before inventory    | No activation cluster or network.                                                                                       |
| Independent expiry  | One-time Cloud Task armed before infrastructure apply for `2026-08-27T14:28:46Z`.                                       |
| Create              | GKE create operation completed from `13:30:37Z` to `13:36:42Z`; workflow completed in 8m28s.                            |
| Ready read-back     | Cluster `RUNNING`; Autopilot, private nodes, Workload Identity, ephemeral labels and dedicated node identity confirmed. |
| Explicit off        | GKE delete operation completed from `13:38:30Z` to `13:42:23Z`; workflow completed in 5m57s.                            |
| Destroyed inventory | Cluster, activation VPC/subnets, forwarding rules, activation disks and reserved addresses absent.                      |
| Residual check      | Activation state and expiry queue empty; retained foundation zero-drift; no managed trust key present.                  |

The cluster existed for approximately 11m53s from its recorded creation time
to completion of deletion. It was left `RUNNING` only for the short evidence
window before `off` began.

Protected on/off artifacts retained the before, created and destroyed
inventories, exact activation inputs, plans and the generation-pinned expiry
package. The independently downloaded artifacts contained no credential file;
the on/off activation records and expiry-package hashes matched.

## Security and recovery observations

The final binding used:

- GitHub Workload Identity Federation constrained by immutable owner and
  repository IDs, the exact repository, `main` and the named environment;
- no reusable Google Cloud service-account key;
- separate operator, expiry, teardown and GKE node identities;
- a dedicated Autopilot node identity with the standard least-privilege GKE
  node role, rather than the project default Compute service account;
- a task armed before activation apply; and
- inventory-authoritative off, with the task removed only after cluster and
  activation-network absence was proven.

Preliminary guarded attempts exposed integration defects before the completed
run: workflow choice parsing, an outdated OIDC-subject assumption, WIF
credential context, operational task read/delete rights and implicit default
node identity. Each stopped or partial attempt was reconciled before the next.
One partial attempt created only the VPC/subnet; the ordinary `off` definition
destroyed both and independent inventory proved no residue. These failures are
useful fail-closed and recovery evidence, not successful activation evidence.

The current private-repository GitHub plan did not permit a required-reviewer
rule for the environment. `main` is the environment's only deployment branch,
the cloud federation repeats the repository/ref/environment restriction, and
repository workflow authority controls dispatch. The result must not be
described as second-person approval.

## Cost evidence

The reviewed one-hour forecast used the official
[GKE cluster-management list price](https://cloud.google.com/kubernetes-engine/pricing)
of USD 0.10 per cluster-hour before credits. Applying that rate to the observed
11m53s cluster lifetime gives an indicative management-fee amount of about USD
0.02 before credit. This is forecast-derived, not an invoiced actual.

Billing was enabled and an existing budget was confirmed. Gross actual,
credits and net billed cost remain pending because provider billing data can
lag. Small retained storage, registry, logging and lifecycle-operation costs
also require delayed read-back. Credits were not treated as zero cost or as an
off mechanism.

## Remaining gates

This evidence proves short-lived operator federation, expiry setup, exact
infrastructure create, manual off, partial-failure recovery and conclusive
residual inventory. The scheduled task did not fire because manual off
completed first, so expiry-triggered Cloud Build execution remains an M3.5
evidence case.

M3.3 must still deploy the local-first runtime walking skeleton and prove
artifact and contract parity. M3.4 must still implement managed trust,
presenter and workload identity, protected state and shared-use security.
M3.5 must run the repeatable scenario and exercise scheduled automatic expiry.
None of those claims follows from this infrastructure spike.

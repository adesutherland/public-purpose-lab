# Gate C governed source-intake guide

Status: In-development operator guide for the complete DS-03 Gate C path

This guide runs the local synthetic-only source-intake, validation, staged
release and basic `KNO-01` processing transaction. It proves upload and paste
semantics, authenticated submission, immutable quarantine, bounded visible
validation, an exact `AUT-01` decision, idempotent processing, restart
reconciliation and metadata-only lifecycle events. It does not prove RAG
ingestion, semantic understanding, evidence quality, malware clearance,
compliance or production readiness.

## Start the local Kubernetes environment

From the repository root:

```sh
PPL_MINIKUBE_PROFILE=public-purpose-lab-gate-a \
PPL_M3_ENVIRONMENT_DIRECTORY=/Users/adrian/CLionProjects/public-purpose-lab-demonstration-requirements/.local/gate-a-minikube \
deploy/local/start-m3-minikube.sh
```

The start command retains the environment-specific synthetic root and existing
component state. It builds the current application image, reconciles NATS
permissions and deploys separate `CNT-01` and `KNO-01` persistent volumes. Use another
profile and environment-directory pair when deliberately creating an isolated
environment.

In three terminals, expose the user-facing services:

```sh
kubectl -n public-purpose-lab port-forward service/m3-scenario-director 18081:8080
```

```sh
kubectl -n public-purpose-lab port-forward service/m3-presentation-gateway 18082:8080
```

```sh
kubectl -n public-purpose-lab port-forward service/gate-a-operations 18084:8080
```

Open:

- Director: <http://localhost:18081/>
- Presentation: <http://presentation.localhost:18082/>
- Workbench: <http://workbench.localhost:18082/workbench/>
- Operations: <http://localhost:18084/>

## Walk the current increment

1. In the Director, connect the local test presenter and create a session.
2. Connect and register both the Presentation and Workbench surfaces to that
   session.
3. Prepare and start the scenario.
4. Establish `synthetic-reviewer` on `reviewer-workbench` from the Director,
   then refresh identity in the Workbench.
5. Ask the Director to show `WB-SOURCE-INTAKE`, or open **Source intake** from
   ordinary Workbench navigation.
6. Upload a small `.txt` or `.md` file, or paste synthetic text. Review the
   preview and supply title, owner, rights and provenance.
7. Confirm synthetic classification and select **Submit to quarantine**.
8. Inspect `WB-SOURCE-STATUS`. It must show quarantine, immutable version 1,
   digest, actor and correlation plus five conclusive validation checks; it
   must not return the source body.
9. Select **Release validated source to staging**. The Workbench must show
   `staged`, the named synthetic reviewer, safe reason and `AUT-01` policy
   decision reference. This is a staging decision, not source or report
   approval.
10. Watch Workbench record `accepted`, `processing` and exactly one terminal
    `completed` or `failed` result. A completed fixture shows digest
    verification, byte/line/section counts and a maximum 240-character safe
    preview. These are deterministic inspection results, not meaning or
    evidence quality.
11. Ask the Director to show `WB-SOURCE-STATUS` and `PRES-PROGRESS`.
    Presentation shows component, lifecycle, timing and counts only. It gains
    no control, source body or preview.
12. In Operations, select the Demonstration Session correlation and inspect
    `CNT-01` followed by `KNO-01`: `source.received`,
    `source.quarantined`, `source.validated` and `source.staged` under the same
    correlation identifier, followed by `processing.accepted`,
    `processing.started` and one terminal event. Safe event and source-version
    identifiers provide evidence references; `AUT-01` remains separately
    deployed.
13. Restart `gate-a-knowledge-processing`, wait for readiness, and refresh the
    same Workbench status. The processing identifier and terminal count must
    remain unchanged.

The link field is deliberately disabled: Gate C does not fetch remote content.
A completed processing result is not indexed, retrieved, understood, treated as
evidence-quality assured or approved for a business decision.

## Automated system check

With the three services reachable at the ports above:

```sh
PPL_DIRECTOR_ORIGIN=http://localhost:18081 \
PPL_GATEWAY_ORIGIN=http://presentation.localhost:18082 \
PPL_WORKBENCH_ORIGIN=http://workbench.localhost:18082 \
PPL_OPERATIONS_URL=http://127.0.0.1:18084 \
tools/smoke-m3-native.sh
```

This includes visible validation, authorised staging, paste and upload
processing, a disclosed synthetic failure, exact retry, empty/malformed/
oversized/hostile refusal, source-body disclosure checks, Operations sequence
checks and KNO-01 restart reconciliation. `deploy/local/smoke-m3-minikube.sh`
starts temporary port-forwards, requests the restart, and runs this check plus
the complete component-mesh smoke when the standard ports are free.

## Stop and retain state

Stop the three foreground port-forward commands with Control-C. To suspend the
local cluster while retaining its profile and volumes:

```sh
minikube stop --profile public-purpose-lab-gate-a
```

Delete neither the profile nor the environment directory when the state or
environment-local trust domain must be retained. Backup and restore are not yet
qualified for this Gate C baseline.

# Gate C governed source-intake guide

Status: In-development operator guide for the first DS-03 increment

This guide runs the local synthetic-only source-intake transaction. It proves
upload or paste preview, authenticated submission, immutable quarantine,
visible source status and metadata-only lifecycle events. It does not yet prove
validation, staging, knowledge processing or Gate C closure.

## Start the local Kubernetes environment

From the repository root:

```sh
PPL_MINIKUBE_PROFILE=public-purpose-lab \
PPL_M3_ENVIRONMENT_DIRECTORY=.local/m3-minikube \
deploy/local/start-m3-minikube.sh
```

The start command retains the environment-specific synthetic root and existing
component state. It builds the current application image, reconciles NATS
permissions and deploys a dedicated `CNT-01` persistent volume. Use another
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
   digest, actor and correlation; it must not return the source body.
9. In Operations, filter for `CNT-01`. Inspect `source.received` and
   `source.quarantined` under the same correlation identifier.

The link field is deliberately disabled: this increment does not fetch remote
content. A quarantined source is not validated, staged, indexed or approved.

## Automated system check

With the three services reachable at the ports above:

```sh
PPL_DIRECTOR_ORIGIN=http://localhost:18081 \
PPL_GATEWAY_ORIGIN=http://presentation.localhost:18082 \
PPL_WORKBENCH_ORIGIN=http://workbench.localhost:18082 \
PPL_OPERATIONS_URL=http://127.0.0.1:18084 \
tools/smoke-m3-native.sh
```

This includes exact-retry reconciliation, changed-idempotency refusal and
Operations event checks. `deploy/local/smoke-m3-minikube.sh` starts temporary
port-forwards and runs this check plus the complete component-mesh smoke when
the standard ports are free.

## Stop and retain state

Stop the three foreground port-forward commands with Control-C. To suspend the
local cluster while retaining its profile and volumes:

```sh
minikube stop --profile public-purpose-lab
```

Delete neither the profile nor the environment directory when the state or
environment-local trust domain must be retained. Backup and restore are not yet
qualified for this Gate C increment.

#!/bin/sh
# shellcheck disable=SC2016 # $JS is a literal NATS JetStream subject prefix.
set -eu

profile=${PPL_MINIKUBE_PROFILE:-public-purpose-lab}
namespace=public-purpose-lab
environment_directory=${PPL_M3_ENVIRONMENT_DIRECTORY:-.local/m3-minikube}
image=public-purpose-lab/m3-runtime:development

require_command() {
  command -v "$1" >/dev/null 2>&1 || {
    printf 'required-command-unavailable:%s\n' "$1" >&2
    exit 1
  }
}

for command in minikube kubectl helm jq nsc openssl; do
  require_command "$command"
done

if [ "$(minikube status --profile "$profile" --format '{{.Host}}' 2>/dev/null || true)" != "Running" ]; then
  minikube start --profile "$profile" --driver qemu2 --container-runtime containerd
fi
if [ ! -f "$environment_directory/root-ca.key" ]; then
  PPL_LOCAL_NATS_PORT=4222 deploy/local/setup-m3-environment.sh \
    "$environment_directory" portable
fi
for required_file in identity.seed identity.nkey; do
  if [ ! -f "$environment_directory/$required_file" ]; then
    printf '%s\n' \
      'pre-m3-4-environment-refused: choose a new PPL_M3_ENVIRONMENT_DIRECTORY; existing trust material was not changed.' >&2
    exit 2
  fi
done

build_log=$(mktemp)
trap 'rm -f "$build_log"' EXIT
minikube image build --profile "$profile" \
  --file deploy/containers/m3-runtime.Containerfile \
  --tag "$image" . >"$build_log" 2>&1
sed -n '1,240p' "$build_log"
if grep -Eq '(^#[0-9]+ ERROR:|^ERROR: failed to solve)' "$build_log"; then
  printf 'minikube-image-build-failed\n' >&2
  exit 1
fi
rm -f "$build_log"
trap - EXIT
image_inventory=$(minikube ssh --profile "$profile" -- \
  sudo crictl images --digests --no-trunc --output json)
image_digest=$(printf '%s' "$image_inventory" | jq -r \
  --arg image "$image" \
  '[.images[] | select(any(.repoTags[]?; endswith($image))) | (.repoDigests[0] // .id)][0] // empty')
[ -n "$image_digest" ] || {
  printf 'built-image-digest-unavailable\n' >&2
  exit 1
}

kubectl apply -f deploy/kubernetes/m3/base/namespace.yaml
kubectl --namespace "$namespace" create secret generic ppl-m3-root-ca \
  --from-file=ca.crt="$environment_directory/root-ca.crt" \
  --dry-run=client --output yaml | kubectl apply -f -
kubectl --namespace "$namespace" create secret tls ppl-m3-nats-tls \
  --cert="$environment_directory/nats-server.crt" \
  --key="$environment_directory/nats-server.key" \
  --dry-run=client --output yaml | kubectl apply -f -
kubectl --namespace "$namespace" create secret generic ppl-m3-director-nkey \
  --from-file=director.seed="$environment_directory/director.seed" \
  --from-file=workload.nkey="$environment_directory/director.nkey" \
  --dry-run=client --output yaml | kubectl apply -f -
kubectl --namespace "$namespace" create secret generic ppl-m3-presentation-nkey \
  --from-file=presentation.seed="$environment_directory/presentation.seed" \
  --from-file=workload.nkey="$environment_directory/presentation.nkey" \
  --dry-run=client --output yaml | kubectl apply -f -
kubectl --namespace "$namespace" create secret generic ppl-m3-identity-nkey \
  --from-file=identity.seed="$environment_directory/identity.seed" \
  --from-file=workload.nkey="$environment_directory/identity.nkey" \
  --dry-run=client --output yaml | kubectl apply -f -
for workload in authorisation engagement source-governance knowledge-processing review-workflow reporting audit-evidence operations event-infrastructure; do
  kubectl --namespace "$namespace" create secret generic "ppl-gate-a-$workload-nkey" \
    --from-file=component.seed="$environment_directory/$workload.seed" \
    --from-file=workload.nkey="$environment_directory/$workload.nkey" \
    --dry-run=client --output yaml | kubectl apply -f -
done

director_public=$(sed -n '1p' "$environment_directory/director.nkey")
presentation_public=$(sed -n '1p' "$environment_directory/presentation.nkey")
identity_public=$(sed -n '1p' "$environment_directory/identity.nkey")
authorisation_public=$(sed -n '1p' "$environment_directory/authorisation.nkey")
engagement_public=$(sed -n '1p' "$environment_directory/engagement.nkey")
source_governance_public=$(sed -n '1p' "$environment_directory/source-governance.nkey")
knowledge_processing_public=$(sed -n '1p' "$environment_directory/knowledge-processing.nkey")
review_workflow_public=$(sed -n '1p' "$environment_directory/review-workflow.nkey")
reporting_public=$(sed -n '1p' "$environment_directory/reporting.nkey")
audit_evidence_public=$(sed -n '1p' "$environment_directory/audit-evidence.nkey")
operations_public=$(sed -n '1p' "$environment_directory/operations.nkey")
event_infrastructure_public=$(sed -n '1p' "$environment_directory/event-infrastructure.nkey")
helm upgrade --install nats nats/nats \
  --version 2.14.5 \
  --namespace "$namespace" \
  --values deploy/kubernetes/m3/nats-values.yaml \
  --set-string "config.merge.authorization.users[0].nkey=$director_public" \
  --set-json 'config.merge.authorization.users[0].permissions.publish=["ppl.m3.to-presentation.*","ppl.m3.to-identity.*","ppl.m3.events.director","ppl.gate-a.events.CTL-01","$JS.>"]' \
  --set-json 'config.merge.authorization.users[0].permissions.subscribe=["ppl.m3.to-director.*","_INBOX.>"]' \
  --set-string "config.merge.authorization.users[1].nkey=$presentation_public" \
  --set-json 'config.merge.authorization.users[1].permissions.publish=["ppl.m3.to-director.*","ppl.gate-a.events.CTL-02","ppl.gate-c.commands.CNT-01","ppl.gate-c.queries.CNT-01","$JS.>"]' \
  --set-json 'config.merge.authorization.users[1].permissions.subscribe=["ppl.m3.to-presentation.*","_INBOX.>"]' \
  --set-string "config.merge.authorization.users[2].nkey=$identity_public" \
  --set-json 'config.merge.authorization.users[2].permissions.publish=["ppl.m3.to-presentation.synthetic-grant","ppl.m3.to-director.identity-outcome","ppl.gate-a.events.IAM-01","$JS.>"]' \
  --set-json 'config.merge.authorization.users[2].permissions.subscribe=["ppl.m3.to-identity.*","_INBOX.>"]' \
  --set-string "config.merge.authorization.users[3].nkey=$authorisation_public" \
  --set-json 'config.merge.authorization.users[3].permissions.publish=["ppl.gate-a.events.AUT-01","_INBOX.>"]' \
  --set-json 'config.merge.authorization.users[3].permissions.subscribe=["ppl.gate-a.commands.AUT-01","ppl.gate-c.decisions.AUT-01","_INBOX.>"]' \
  --set-string "config.merge.authorization.users[4].nkey=$engagement_public" \
  --set-json 'config.merge.authorization.users[4].permissions.publish=["ppl.gate-a.events.DOM-01"]' \
  --set-json 'config.merge.authorization.users[4].permissions.subscribe=["ppl.gate-a.commands.DOM-01","_INBOX.>"]' \
  --set-string "config.merge.authorization.users[5].nkey=$source_governance_public" \
  --set-json 'config.merge.authorization.users[5].permissions.publish=["ppl.gate-a.events.CNT-01","ppl.gate-c.events.CNT-01","ppl.gate-c.decisions.AUT-01","_INBOX.>"]' \
  --set-json 'config.merge.authorization.users[5].permissions.subscribe=["ppl.gate-a.commands.CNT-01","ppl.gate-c.commands.CNT-01","ppl.gate-c.queries.CNT-01","_INBOX.>"]' \
  --set-string "config.merge.authorization.users[6].nkey=$knowledge_processing_public" \
  --set-json 'config.merge.authorization.users[6].permissions.publish=["ppl.gate-a.events.KNO-01"]' \
  --set-json 'config.merge.authorization.users[6].permissions.subscribe=["ppl.gate-a.commands.KNO-01","_INBOX.>"]' \
  --set-string "config.merge.authorization.users[7].nkey=$review_workflow_public" \
  --set-json 'config.merge.authorization.users[7].permissions.publish=["ppl.gate-a.events.WRK-01"]' \
  --set-json 'config.merge.authorization.users[7].permissions.subscribe=["ppl.gate-a.commands.WRK-01","_INBOX.>"]' \
  --set-string "config.merge.authorization.users[8].nkey=$reporting_public" \
  --set-json 'config.merge.authorization.users[8].permissions.publish=["ppl.gate-a.events.RPT-01"]' \
  --set-json 'config.merge.authorization.users[8].permissions.subscribe=["ppl.gate-a.commands.RPT-01","_INBOX.>"]' \
  --set-string "config.merge.authorization.users[9].nkey=$audit_evidence_public" \
  --set-json 'config.merge.authorization.users[9].permissions.publish=["ppl.gate-a.events.AUD-01"]' \
  --set-json 'config.merge.authorization.users[9].permissions.subscribe=["ppl.gate-a.commands.AUD-01","_INBOX.>"]' \
  --set-string "config.merge.authorization.users[10].nkey=$operations_public" \
  --set-json 'config.merge.authorization.users[10].permissions.publish=["ppl.gate-a.events.OPS-01","ppl.gate-a.commands.*"]' \
  --set-json 'config.merge.authorization.users[10].permissions.subscribe=["ppl.gate-a.commands.OPS-01","ppl.gate-a.events.*","_INBOX.>"]' \
  --set-string "config.merge.authorization.users[11].nkey=$event_infrastructure_public" \
  --set-json 'config.merge.authorization.users[11].permissions.publish=["ppl.gate-a.events.INT-01"]' \
  --set-json 'config.merge.authorization.users[11].permissions.subscribe=["ppl.gate-a.commands.INT-01","_INBOX.>"]' \
  --wait --timeout 5m
kubectl --namespace "$namespace" rollout restart statefulset/nats
kubectl --namespace "$namespace" rollout status statefulset/nats --timeout=5m

source_revision=$(git rev-parse HEAD)
if [ -n "$(git status --short)" ]; then
  source_revision="$source_revision+working-tree"
fi
kubectl --namespace "$namespace" create configmap ppl-m3-build-evidence \
  --from-literal=sourceRevision="$source_revision" \
  --from-literal=imageDigest="$image_digest" \
  --dry-run=client --output yaml | kubectl apply -f -
kubectl --namespace "$namespace" create configmap ppl-m3-environment \
  --from-literal=environmentId="$(sed -n '1p' "$environment_directory/environment-id")" \
  --dry-run=client --output yaml | kubectl apply -f -
kubectl apply -k deploy/kubernetes/m3/base
helm upgrade --install gate-a-components deploy/kubernetes/gate-a \
  --namespace "$namespace" \
  --wait --timeout 5m
kubectl --namespace "$namespace" rollout restart \
  deployment/m3-identity-broker deployment/m3-scenario-director deployment/m3-presentation-gateway \
  deployment/gate-a-authorisation deployment/gate-a-engagement \
  deployment/gate-a-source-governance deployment/gate-a-knowledge-processing \
  deployment/gate-a-review-workflow deployment/gate-a-reporting \
  deployment/gate-a-audit-evidence deployment/gate-a-operations \
  deployment/gate-a-event-infrastructure
kubectl --namespace "$namespace" rollout status deployment/m3-identity-broker --timeout=5m
kubectl --namespace "$namespace" rollout status deployment/m3-scenario-director --timeout=5m
kubectl --namespace "$namespace" rollout status deployment/m3-presentation-gateway --timeout=5m
for workload in authorisation engagement source-governance knowledge-processing review-workflow reporting audit-evidence operations event-infrastructure; do
  kubectl --namespace "$namespace" rollout status "deployment/gate-a-$workload" --timeout=5m
done
for workload in m3-identity-broker m3-scenario-director m3-presentation-gateway gate-a-authorisation gate-a-engagement gate-a-source-governance gate-a-knowledge-processing gate-a-review-workflow gate-a-reporting gate-a-audit-evidence gate-a-operations gate-a-event-infrastructure; do
  running_image=$(kubectl --namespace "$namespace" get pods \
    --selector "app.kubernetes.io/name=$workload" \
    --output jsonpath='{.items[0].status.containerStatuses[0].imageID}')
  case "$running_image" in
    *"$image_digest") ;;
    *)
      printf 'running-image-mismatch:%s\n' "$workload" >&2
      exit 1
      ;;
  esac
done

printf '%s\n' \
  "M3.4 Minikube image: $image_digest" \
  'Identity health port-forward: kubectl -n public-purpose-lab port-forward service/m3-identity-broker 18083:8080' \
  'Director port-forward: kubectl -n public-purpose-lab port-forward service/m3-scenario-director 18081:8080' \
  'Gateway port-forward: kubectl -n public-purpose-lab port-forward service/m3-presentation-gateway 18082:8080' \
  'Operations port-forward: kubectl -n public-purpose-lab port-forward service/gate-a-operations 18084:8080' \
  'The deployment has no Ingress or LoadBalancer.'

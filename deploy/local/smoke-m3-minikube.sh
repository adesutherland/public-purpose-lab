#!/bin/sh
set -eu

namespace=public-purpose-lab
mkdir -p .local/m3-minikube-run
kubectl --namespace "$namespace" port-forward service/m3-scenario-director \
  18081:8080 >.local/m3-minikube-run/director-port-forward.log 2>&1 &
director_forward_pid=$!
kubectl --namespace "$namespace" port-forward service/m3-presentation-gateway \
  18082:8080 >.local/m3-minikube-run/presentation-port-forward.log 2>&1 &
presentation_forward_pid=$!
kubectl --namespace "$namespace" port-forward service/gate-a-operations \
  18084:8080 >.local/m3-minikube-run/operations-port-forward.log 2>&1 &
operations_forward_pid=$!
trap 'kill "$director_forward_pid" "$presentation_forward_pid" "$operations_forward_pid" 2>/dev/null || true' EXIT

attempt=0
while [ "$attempt" -lt 40 ]; do
  if curl -fsS http://127.0.0.1:18081/health/ready >/dev/null 2>&1 \
    && curl -fsS http://127.0.0.1:18082/health/ready >/dev/null 2>&1 \
    && curl -fsS http://127.0.0.1:18084/health/ready >/dev/null 2>&1; then
    break
  fi
  attempt=$((attempt + 1))
  sleep 0.25
done
[ "$attempt" -lt 40 ] || {
  printf 'minikube-port-forward-readiness-failed\n' >&2
  exit 1
}

PPL_DIRECTOR_ORIGIN=http://localhost:18081 \
  PPL_GATEWAY_ORIGIN=http://presentation.localhost:18082 \
  PPL_WORKBENCH_ORIGIN=http://workbench.localhost:18082 \
  PPL_OPERATIONS_URL=http://127.0.0.1:18084 \
  PPL_KNO_RESTART_MODE=minikube \
  PPL_SSE_CAPTURE=.local/m3-minikube-run/sse.txt \
  tools/smoke-m3-native.sh
tools/smoke-gate-a.sh

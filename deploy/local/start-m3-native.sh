#!/bin/sh
set -eu

environment_directory=${1:-.local/m3-environment}
nats_port=${PPL_LOCAL_NATS_PORT:-4223}

if [ ! -f "$environment_directory/nats-server.conf" ]; then
  printf '%s\n' 'M3 environment is absent; run deploy/local/setup-m3-environment.sh first.' >&2
  exit 2
fi
for required_file in identity.seed identity.nkey; do
  if [ ! -f "$environment_directory/$required_file" ]; then
    printf '%s\n' \
      'pre-m3-4-environment-refused: create a new environment directory; existing trust material was not changed.' >&2
    exit 2
  fi
done
identity_public=$(sed -n '1p' "$environment_directory/identity.nkey")
if [ -z "$identity_public" ] || ! grep -Fq "$identity_public" "$environment_directory/nats-server.conf"; then
  printf '%s\n' \
    'pre-m3-4-environment-refused: Identity Broker authority is absent; existing trust material was not changed.' >&2
  exit 2
fi

mkdir -p "$environment_directory/state" "$environment_directory/run"
environment_id=$(sed -n '1p' "$environment_directory/environment-id")

nats-server -c "$environment_directory/nats-server.conf" \
  >"$environment_directory/run/nats.log" 2>&1 &
printf '%s\n' "$!" >"$environment_directory/run/nats.pid"

attempt=0
while [ "$attempt" -lt 40 ]; do
  if grep -q 'Server is ready' "$environment_directory/run/nats.log"; then
    break
  fi
  attempt=$((attempt + 1))
  sleep 0.1
done
[ "$attempt" -lt 40 ] || {
  printf '%s\n' 'M3 NATS broker did not become ready.' >&2
  exit 1
}

PPL_RUNTIME_MODE=identity-broker \
PPL_RUNTIME_PROFILE=native-development \
PPL_LISTEN_ADDRESS=127.0.0.1:18083 \
PPL_ALLOWED_ORIGIN=http://127.0.0.1:18083 \
PPL_STATE_PATH="$environment_directory/state/identity-runtime.sqlite" \
PPL_IDENTITY_STATE_ROOT="$environment_directory/state/identity" \
PPL_TRUST_BUNDLE_PATH="$environment_directory/identity-public/trust-bundle.json" \
PPL_ENVIRONMENT_ID="$environment_id" \
PPL_NATS_URL="tls://127.0.0.1:$nats_port" \
PPL_NATS_NKEY_SEED_FILE="$environment_directory/identity.seed" \
PPL_NATS_ROOT_CERTIFICATE="$environment_directory/root-ca.crt" \
target/debug/ppl-m3-runtime >"$environment_directory/run/identity.log" 2>&1 &
printf '%s\n' "$!" >"$environment_directory/run/identity.pid"

attempt=0
while [ "$attempt" -lt 40 ]; do
  if curl -fsS http://127.0.0.1:18083/health/ready >/dev/null 2>&1; then
    break
  fi
  attempt=$((attempt + 1))
  sleep 0.1
done
[ "$attempt" -lt 40 ] || {
  printf '%s\n' 'M3 identity broker did not become ready.' >&2
  exit 1
}

PPL_RUNTIME_MODE=scenario-director \
PPL_RUNTIME_PROFILE=native-development \
PPL_LISTEN_ADDRESS=127.0.0.1:18081 \
PPL_ALLOWED_ORIGIN=http://127.0.0.1:18081 \
PPL_ADDITIONAL_ALLOWED_ORIGINS=http://localhost:18081 \
PPL_PRESENTATION_SURFACE_URL=http://presentation.localhost:18082/ \
PPL_WORKBENCH_SURFACE_URL=http://workbench.localhost:18082/workbench/ \
PPL_OPERATIONS_SURFACE_URL=http://127.0.0.1:18084/ \
PPL_STATE_PATH="$environment_directory/state/ctl-01.sqlite" \
PPL_SECURITY_STATE_PATH="$environment_directory/state/director-security.sqlite" \
PPL_ENVIRONMENT_ID="$environment_id" \
PPL_NATS_URL="tls://127.0.0.1:$nats_port" \
PPL_NATS_NKEY_SEED_FILE="$environment_directory/director.seed" \
PPL_NATS_ROOT_CERTIFICATE="$environment_directory/root-ca.crt" \
target/debug/ppl-m3-runtime >"$environment_directory/run/director.log" 2>&1 &
printf '%s\n' "$!" >"$environment_directory/run/director.pid"

PPL_RUNTIME_MODE=presentation-gateway \
PPL_RUNTIME_PROFILE=native-development \
PPL_LISTEN_ADDRESS=127.0.0.1:18082 \
PPL_ALLOWED_ORIGIN=http://presentation.localhost:18082 \
PPL_ADDITIONAL_ALLOWED_ORIGINS=http://workbench.localhost:18082 \
PPL_STATE_PATH="$environment_directory/state/ctl-02.sqlite" \
PPL_SECURITY_STATE_PATH="$environment_directory/state/presentation-security.sqlite" \
PPL_TRUST_BUNDLE_PATH="$environment_directory/identity-public/trust-bundle.json" \
PPL_ENVIRONMENT_ID="$environment_id" \
PPL_NATS_URL="tls://127.0.0.1:$nats_port" \
PPL_NATS_NKEY_SEED_FILE="$environment_directory/presentation.seed" \
PPL_NATS_ROOT_CERTIFICATE="$environment_directory/root-ca.crt" \
target/debug/ppl-m3-runtime >"$environment_directory/run/presentation.log" 2>&1 &
printf '%s\n' "$!" >"$environment_directory/run/presentation.pid"

for endpoint in \
  'Director:http://127.0.0.1:18081/health/ready' \
  'Presentation Gateway:http://127.0.0.1:18082/health/ready'; do
  name=${endpoint%%:*}
  url=${endpoint#*:}
  attempt=0
  while [ "$attempt" -lt 60 ]; do
    if curl -fsS "$url" >/dev/null 2>&1; then
      break
    fi
    attempt=$((attempt + 1))
    sleep 0.1
  done
  if [ "$attempt" -ge 60 ]; then
    deploy/local/stop-m3-native.sh "$environment_directory" >/dev/null 2>&1 || true
    printf '%s\n' "M3 $name did not become ready; started processes were stopped." >&2
    exit 1
  fi
done

printf '%s\n' \
  'Director: http://localhost:18081/' \
  'Audience display: http://presentation.localhost:18082/' \
  'Workbench surface: http://workbench.localhost:18082/workbench/'

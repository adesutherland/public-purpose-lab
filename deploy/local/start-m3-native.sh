#!/bin/sh
set -eu

environment_directory=${1:-.local/m3-environment}
nats_port=${PPL_LOCAL_NATS_PORT:-4223}

if [ ! -f "$environment_directory/nats-server.conf" ]; then
  printf '%s\n' 'M3 environment is absent; run deploy/local/setup-m3-environment.sh first.' >&2
  exit 2
fi

mkdir -p "$environment_directory/state" "$environment_directory/run"

nats-server -c "$environment_directory/nats-server.conf" \
  >"$environment_directory/run/nats.log" 2>&1 &
printf '%s\n' "$!" >"$environment_directory/run/nats.pid"

PPL_RUNTIME_MODE=scenario-director \
PPL_RUNTIME_PROFILE=native-development \
PPL_LISTEN_ADDRESS=127.0.0.1:18081 \
PPL_ALLOWED_ORIGIN=http://127.0.0.1:18081 \
PPL_STATE_PATH="$environment_directory/state/ctl-01.sqlite" \
PPL_NATS_URL="tls://127.0.0.1:$nats_port" \
PPL_NATS_NKEY_SEED_FILE="$environment_directory/director.seed" \
PPL_NATS_ROOT_CERTIFICATE="$environment_directory/root-ca.crt" \
target/debug/ppl-m3-runtime >"$environment_directory/run/director.log" 2>&1 &
printf '%s\n' "$!" >"$environment_directory/run/director.pid"

PPL_RUNTIME_MODE=presentation-gateway \
PPL_RUNTIME_PROFILE=native-development \
PPL_LISTEN_ADDRESS=127.0.0.1:18082 \
PPL_ALLOWED_ORIGIN=http://127.0.0.1:18082 \
PPL_STATE_PATH="$environment_directory/state/ctl-02.sqlite" \
PPL_NATS_URL="tls://127.0.0.1:$nats_port" \
PPL_NATS_NKEY_SEED_FILE="$environment_directory/presentation.seed" \
PPL_NATS_ROOT_CERTIFICATE="$environment_directory/root-ca.crt" \
target/debug/ppl-m3-runtime >"$environment_directory/run/presentation.log" 2>&1 &
printf '%s\n' "$!" >"$environment_directory/run/presentation.pid"

printf '%s\n' \
  'Director: http://127.0.0.1:18081/' \
  'Audience display: http://127.0.0.1:18082/' \
  'Workbench surface: http://127.0.0.1:18082/workbench/'

#!/bin/sh
set -eu

operations_url=${PPL_OPERATIONS_URL:-http://127.0.0.1:18084}
snapshot_file=$(mktemp)
events_file=$(mktemp)
probe_file=$(mktemp)
trap 'rm -f "$snapshot_file" "$events_file" "$probe_file"' EXIT

attempt=0
while [ "$attempt" -lt 80 ]; do
  if curl -fsS "$operations_url/api/v1/mesh" >"$snapshot_file" 2>/dev/null \
    && node -e '
      const fs = require("node:fs");
      const value = JSON.parse(fs.readFileSync(process.argv[1], "utf8"));
      const identities = value.components.map((component) => component.workloadIdentity);
      if (value.expected !== 12 || value.ready !== 12 || value.status !== "ready") process.exit(1);
      if (new Set(identities).size !== 12 || identities.some((identity) => !identity)) process.exit(1);
    ' "$snapshot_file"; then
    break
  fi
  attempt=$((attempt + 1))
  sleep 0.25
done
[ "$attempt" -lt 80 ] || {
  printf '%s\n' 'gate-a-component-readiness-failed' >&2
  test ! -s "$snapshot_file" || sed -n '1,80p' "$snapshot_file" >&2
  exit 1
}

curl -fsS -X POST "$operations_url/api/v1/probe" >"$probe_file"
correlation_id=$(node -e '
  const fs = require("node:fs");
  const value = JSON.parse(fs.readFileSync(process.argv[1], "utf8"));
  if (value.status !== "issued" || value.targetCount !== 9) process.exit(1);
  process.stdout.write(value.correlationId);
' "$probe_file")

attempt=0
while [ "$attempt" -lt 40 ]; do
  curl -fsS "$operations_url/api/v1/events" >"$events_file"
  if node -e '
    const fs = require("node:fs");
    const value = JSON.parse(fs.readFileSync(process.argv[1], "utf8"));
    const events = value.events.filter((event) => event.correlationId === process.argv[2]);
    const accepted = events.filter((event) => event.eventType === "component.command-accepted");
    const components = new Set(accepted.map((event) => event.componentId));
    if (accepted.length !== 9 || components.size !== 9) process.exit(1);
  ' "$events_file" "$correlation_id"; then
    break
  fi
  attempt=$((attempt + 1))
  sleep 0.1
done
[ "$attempt" -lt 40 ] || {
  printf '%s\n' 'gate-a-command-event-path-failed' >&2
  sed -n '1,120p' "$events_file" >&2
  exit 1
}

printf '%s\n' \
  'Gate A component mesh smoke passed.' \
  '12 distinct workloads reported ready.' \
  "9 capability commands concluded under $correlation_id."

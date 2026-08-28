#!/bin/sh
set -eu

environment_directory=${1:-.local/m3-environment}

for process_name in director presentation nats; do
  pid_file="$environment_directory/run/$process_name.pid"
  if [ -f "$pid_file" ]; then
    pid=$(sed -n '1p' "$pid_file")
    case "$pid" in
      ''|*[!0-9]*) printf '%s\n' "Ignoring invalid PID file: $pid_file" >&2 ;;
      *) kill -TERM "$pid" 2>/dev/null || true ;;
    esac
  fi
done

printf '%s\n' 'M3.3 native processes stopped; component and broker state was retained.'

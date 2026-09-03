#!/bin/sh
# shellcheck disable=SC2016 # $JS is a literal NATS JetStream subject prefix.
set -eu

environment_directory=${1:-.local/m3-environment}
layout=${2:-native}
nats_port=${PPL_LOCAL_NATS_PORT:-}

if [ -e "$environment_directory/root-ca.key" ]; then
  printf '%s\n' 'M3 environment already exists; refusing to rotate its trust material.' >&2
  exit 2
fi

case "$layout" in
  native)
    nats_port=${nats_port:-4223}
    listen_address=127.0.0.1
    store_directory="$environment_directory/nats-data"
    certificate_directory="$environment_directory"
    ;;
  portable)
    nats_port=${nats_port:-4222}
    listen_address=0.0.0.0
    store_directory=/data
    certificate_directory=/etc/nats
    ;;
  *)
    printf '%s\n' 'Environment layout must be native or portable.' >&2
    exit 2
    ;;
esac

umask 077
mkdir -p "$environment_directory" "$environment_directory/nats-data"
chmod 700 "$environment_directory"

pair_file=$(mktemp "$environment_directory/nkey-pair.XXXXXX")
nsc generate nkey --user >"$pair_file"
sed -n '1p' "$pair_file" >"$environment_directory/director.seed"
director_public=$(sed -n '2p' "$pair_file")
printf '%s\n' "$director_public" >"$environment_directory/director.nkey"
nsc generate nkey --user >"$pair_file"
sed -n '1p' "$pair_file" >"$environment_directory/presentation.seed"
presentation_public=$(sed -n '2p' "$pair_file")
printf '%s\n' "$presentation_public" >"$environment_directory/presentation.nkey"
nsc generate nkey --user >"$pair_file"
sed -n '1p' "$pair_file" >"$environment_directory/identity.seed"
identity_public=$(sed -n '2p' "$pair_file")
printf '%s\n' "$identity_public" >"$environment_directory/identity.nkey"
for workload in authorisation engagement source-governance knowledge-processing review-workflow reporting audit-evidence operations event-infrastructure; do
  nsc generate nkey --user >"$pair_file"
  sed -n '1p' "$pair_file" >"$environment_directory/$workload.seed"
  sed -n '2p' "$pair_file" >"$environment_directory/$workload.nkey"
done
rm "$pair_file"
printf 'environment-%s\n' "$(openssl rand -hex 16)" >"$environment_directory/environment-id"

openssl req -x509 -newkey rsa:3072 -sha256 -nodes -days 7 \
  -subj '/CN=Public Purpose Lab local synthetic M3 root' \
  -keyout "$environment_directory/root-ca.key" \
  -out "$environment_directory/root-ca.crt" >/dev/null 2>&1
openssl req -newkey rsa:3072 -sha256 -nodes \
  -subj '/CN=localhost' \
  -keyout "$environment_directory/nats-server.key" \
  -out "$environment_directory/nats-server.csr" >/dev/null 2>&1
printf '%s\n' \
  'subjectAltName=DNS:localhost,DNS:nats,DNS:nats.public-purpose-lab.svc,DNS:nats.public-purpose-lab.svc.cluster.local,IP:127.0.0.1' \
  'extendedKeyUsage=serverAuth' \
  >"$environment_directory/nats-server.ext"
openssl x509 -req -sha256 -days 7 \
  -in "$environment_directory/nats-server.csr" \
  -CA "$environment_directory/root-ca.crt" \
  -CAkey "$environment_directory/root-ca.key" \
  -CAcreateserial \
  -extfile "$environment_directory/nats-server.ext" \
  -out "$environment_directory/nats-server.crt" >/dev/null 2>&1

printf '%s\n' \
  "listen: ${listen_address}:${nats_port}" \
  'jetstream {' \
  "  store_dir: \"${store_directory}\"" \
  '  max_mem: 64MB' \
  '  max_file: 128MB' \
  '}' \
  'tls {' \
  "  cert_file: \"${certificate_directory}/nats-server.crt\"" \
  "  key_file: \"${certificate_directory}/nats-server.key\"" \
  "  ca_file: \"${certificate_directory}/root-ca.crt\"" \
  '  verify: false' \
  '  timeout: 2' \
  '}' \
  'authorization {' \
  '  users: [' \
  '    {' \
  "      nkey: \"${director_public}\"" \
  '      permissions: {' \
  '        publish: ["ppl.m3.to-presentation.*", "ppl.m3.to-identity.*", "ppl.m3.events.director", "ppl.gate-a.events.CTL-01", "$JS.>"]' \
  '        subscribe: ["ppl.m3.to-director.*", "_INBOX.>"]' \
  '      }' \
  '    },' \
  '    {' \
  "      nkey: \"${presentation_public}\"" \
  '      permissions: {' \
  '        publish: ["ppl.m3.to-director.*", "ppl.gate-a.events.CTL-02", "ppl.gate-c.commands.CNT-01", "ppl.gate-c.queries.CNT-01", "ppl.gate-c.queries.KNO-01", "$JS.>"]' \
  '        subscribe: ["ppl.m3.to-presentation.*", "_INBOX.>"]' \
  '      }' \
  '    },' \
  '    {' \
  "      nkey: \"${identity_public}\"" \
  '      permissions: {' \
  '        publish: ["ppl.m3.to-presentation.synthetic-grant", "ppl.m3.to-director.identity-outcome", "ppl.gate-a.events.IAM-01", "$JS.>"]' \
  '        subscribe: ["ppl.m3.to-identity.*", "_INBOX.>"]' \
  '      }' \
  '    },' \
  '    {' \
  "      nkey: \"$(sed -n '1p' "$environment_directory/authorisation.nkey")\"" \
  '      permissions: { publish: ["ppl.gate-a.events.AUT-01", "_INBOX.>"], subscribe: ["ppl.gate-a.commands.AUT-01", "ppl.gate-c.decisions.AUT-01", "_INBOX.>"] }' \
  '    },' \
  '    {' \
  "      nkey: \"$(sed -n '1p' "$environment_directory/engagement.nkey")\"" \
  '      permissions: { publish: ["ppl.gate-a.events.DOM-01"], subscribe: ["ppl.gate-a.commands.DOM-01", "_INBOX.>"] }' \
  '    },' \
  '    {' \
  "      nkey: \"$(sed -n '1p' "$environment_directory/source-governance.nkey")\"" \
  '      permissions: { publish: ["ppl.gate-a.events.CNT-01", "ppl.gate-c.events.CNT-01", "ppl.gate-c.decisions.AUT-01", "_INBOX.>"], subscribe: ["ppl.gate-a.commands.CNT-01", "ppl.gate-c.commands.CNT-01", "ppl.gate-c.queries.CNT-01", "ppl.gate-c.processing-input.CNT-01", "_INBOX.>"] }' \
  '    },' \
  '    {' \
  "      nkey: \"$(sed -n '1p' "$environment_directory/knowledge-processing.nkey")\"" \
  '      permissions: { publish: ["ppl.gate-a.events.KNO-01", "ppl.gate-c.events.KNO-01", "ppl.gate-c.processing-input.CNT-01", "_INBOX.>", "$JS.>"], subscribe: ["ppl.gate-a.commands.KNO-01", "ppl.gate-c.queries.KNO-01", "_INBOX.>"] }' \
  '    },' \
  '    {' \
  "      nkey: \"$(sed -n '1p' "$environment_directory/review-workflow.nkey")\"" \
  '      permissions: { publish: ["ppl.gate-a.events.WRK-01"], subscribe: ["ppl.gate-a.commands.WRK-01", "_INBOX.>"] }' \
  '    },' \
  '    {' \
  "      nkey: \"$(sed -n '1p' "$environment_directory/reporting.nkey")\"" \
  '      permissions: { publish: ["ppl.gate-a.events.RPT-01"], subscribe: ["ppl.gate-a.commands.RPT-01", "_INBOX.>"] }' \
  '    },' \
  '    {' \
  "      nkey: \"$(sed -n '1p' "$environment_directory/audit-evidence.nkey")\"" \
  '      permissions: { publish: ["ppl.gate-a.events.AUD-01"], subscribe: ["ppl.gate-a.commands.AUD-01", "_INBOX.>"] }' \
  '    },' \
  '    {' \
  "      nkey: \"$(sed -n '1p' "$environment_directory/operations.nkey")\"" \
  '      permissions: { publish: ["ppl.gate-a.events.OPS-01", "ppl.gate-a.commands.*"], subscribe: ["ppl.gate-a.commands.OPS-01", "ppl.gate-a.events.*", "_INBOX.>"] }' \
  '    },' \
  '    {' \
  "      nkey: \"$(sed -n '1p' "$environment_directory/event-infrastructure.nkey")\"" \
  '      permissions: { publish: ["ppl.gate-a.events.INT-01"], subscribe: ["ppl.gate-a.commands.INT-01", "_INBOX.>"] }' \
  '    }' \
  '  ]' \
  '}' >"$environment_directory/nats-server.conf"

chmod 600 "$environment_directory"/*.key "$environment_directory"/*.seed \
  "$environment_directory/nats-server.conf"
chmod 644 "$environment_directory"/*.crt
chmod 644 "$environment_directory"/*.nkey
chmod 644 "$environment_directory/environment-id"
if [ "$layout" = portable ]; then
  # Compose file-backed secrets preserve host ownership and may ignore declared
  # uid/gid/mode. The parent directory remains owner-only, and only these
  # workload-mounted files become readable inside their intended containers.
  chmod 644 "$environment_directory/nats-server.key" \
    "$environment_directory/director.seed" \
    "$environment_directory/presentation.seed" \
    "$environment_directory/identity.seed" \
    "$environment_directory/authorisation.seed" \
    "$environment_directory/engagement.seed" \
    "$environment_directory/source-governance.seed" \
    "$environment_directory/knowledge-processing.seed" \
    "$environment_directory/review-workflow.seed" \
    "$environment_directory/reporting.seed" \
    "$environment_directory/audit-evidence.seed" \
    "$environment_directory/operations.seed" \
    "$environment_directory/event-infrastructure.seed" \
    "$environment_directory/nats-server.conf"
fi

printf '%s\n' \
  "Environment generated at $environment_directory" \
  "Environment ID: $(sed -n '1p' "$environment_directory/environment-id")" \
  "Layout: $layout" \
  "NATS TLS endpoint: tls://127.0.0.1:$nats_port" \
  "The root private key stays inside this environment and is not an application mount."

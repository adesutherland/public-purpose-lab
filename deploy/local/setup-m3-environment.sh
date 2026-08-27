#!/bin/sh
set -eu

environment_directory=${1:-.local/m3-environment}
layout=${2:-native}
nats_port=${PPL_LOCAL_NATS_PORT:-4223}

if [ -e "$environment_directory/root-ca.key" ]; then
  printf '%s\n' 'M3 environment already exists; refusing to rotate its trust material.' >&2
  exit 2
fi

case "$layout" in
  native)
    listen_address=127.0.0.1
    store_directory="$environment_directory/nats-data"
    certificate_directory="$environment_directory"
    ;;
  portable)
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
rm "$pair_file"

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
  '        publish: ["ppl.m3.to-presentation.*", "ppl.m3.events.director", "$JS.>"]' \
  '        subscribe: ["ppl.m3.to-director.*", "_INBOX.>"]' \
  '      }' \
  '    },' \
  '    {' \
  "      nkey: \"${presentation_public}\"" \
  '      permissions: {' \
  '        publish: ["ppl.m3.to-director.*", "$JS.>"]' \
  '        subscribe: ["ppl.m3.to-presentation.*", "_INBOX.>"]' \
  '      }' \
  '    }' \
  '  ]' \
  '}' >"$environment_directory/nats-server.conf"

chmod 600 "$environment_directory"/*.key "$environment_directory"/*.seed \
  "$environment_directory/nats-server.conf"
chmod 644 "$environment_directory"/*.crt
chmod 644 "$environment_directory"/*.nkey

printf '%s\n' \
  "Environment generated at $environment_directory" \
  "Layout: $layout" \
  "NATS TLS endpoint: tls://127.0.0.1:$nats_port" \
  "The root private key stays inside this environment and is not an application mount."

#!/bin/sh
set -eu

stage='initialisation'
director_cookie_jar=$(mktemp)
gateway_cookie_jar=$(mktemp)
workbench_cookie_jar=$(mktemp)
workbench_sse_capture=$(mktemp)
# shellcheck disable=SC2154 # Values are assigned before the EXIT trap can run.
trap 'result=$?; rm -f "$director_cookie_jar" "$gateway_cookie_jar" "$workbench_cookie_jar" "$workbench_sse_capture"; if [ "$result" -ne 0 ]; then printf "smoke-failed-at=%s\n" "$stage" >&2; fi' EXIT

director_url=${PPL_DIRECTOR_URL:-http://127.0.0.1:18081}
gateway_url=${PPL_GATEWAY_URL:-http://127.0.0.1:18082}
director_origin=${PPL_DIRECTOR_ORIGIN:-$director_url}
gateway_origin=${PPL_GATEWAY_ORIGIN:-$gateway_url}
sse_capture=${PPL_SSE_CAPTURE:-.local/m3-native-sse.txt}

require_command() {
  command -v "$1" >/dev/null 2>&1 || {
    printf 'required-command-unavailable:%s\n' "$1" >&2
    exit 1
  }
}

login() {
  origin=$1
  url=$2
  cookie_jar=$3
  curl -fsS -c "$cookie_jar" -o /dev/null -H "Origin: $origin" \
    -H 'Content-Type: application/json' -X POST \
    "$url/api/v1/development-session" -d '{}'
}

csrf_token() {
  cookie_jar=$1
  awk '$6 == "PPL_CSRF" { print $7; exit }' "$cookie_jar"
}

post_json() {
  cookie_jar=$1
  csrf=$2
  origin=$3
  url=$4
  body=$5
  curl -sS --fail-with-body -b "$cookie_jar" -H "Origin: $origin" \
    -H "X-PPL-CSRF: $csrf" -H 'Content-Type: application/json' -X POST "$url" -d "$body"
}

require_command curl
require_command jq
login "$director_origin" "$director_url" "$director_cookie_jar"
login "$gateway_origin" "$gateway_url" "$gateway_cookie_jar"
login "$gateway_origin" "$gateway_url" "$workbench_cookie_jar"
director_csrf=$(csrf_token "$director_cookie_jar")
gateway_csrf=$(csrf_token "$gateway_cookie_jar")
workbench_csrf=$(csrf_token "$workbench_cookie_jar")
[ -n "$director_csrf" ] && [ -n "$gateway_csrf" ] && [ -n "$workbench_csrf" ] || {
  printf 'application-session-csrf-unavailable\n' >&2
  exit 1
}

stage='csrf-refusal'
csrf_status=$(curl -sS -o /dev/null -w '%{http_code}' -b "$director_cookie_jar" \
  -H "Origin: $director_origin" -H 'X-PPL-CSRF: wrong' \
  -H 'Content-Type: application/json' -X POST "$director_url/api/v1/sessions" -d '{}')
[ "$csrf_status" = 401 ] || { printf 'wrong-csrf-not-refused\n' >&2; exit 1; }

stage='create-session'
created=$(post_json "$director_cookie_jar" "$director_csrf" "$director_origin" \
  "$director_url/api/v1/sessions" '{}')
session_id=$(printf '%s' "$created" | jq -r '.session.sessionId')
revision=$(printf '%s' "$created" | jq -r '.session.revision')

stage='set-logical-time'
body=$(jq -cn --argjson revision "$revision" \
  '{operation:"set",expectedRevision:$revision,logicalInstant:"2030-01-01T09:00:00Z"}')
logical_time=$(post_json "$director_cookie_jar" "$director_csrf" "$director_origin" \
  "$director_url/api/v1/sessions/$session_id/logical-time" "$body")
revision=$(printf '%s' "$logical_time" | jq -r '.sessionRevision')

stage='register-surface'
body=$(jq -cn --arg session "$session_id" \
  '{sessionId:$session,surfaceSlot:"audience-display",surfaceRole:"audience-display"}')
post_json "$gateway_cookie_jar" "$gateway_csrf" "$gateway_origin" \
  "$gateway_url/api/v1/registrations" "$body" >/dev/null

body=$(jq -cn --arg session "$session_id" \
  '{sessionId:$session,surfaceSlot:"reviewer-workbench",surfaceRole:"reviewer-workbench"}')
post_json "$workbench_cookie_jar" "$workbench_csrf" "$gateway_origin" \
  "$gateway_url/api/v1/registrations" "$body" >/dev/null

attempt=0
while [ "$attempt" -lt 20 ]; do
  status=$(curl -fsS -b "$director_cookie_jar" \
    "$director_url/api/v1/status/$session_id")
  [ "$(printf '%s' "$status" | jq -r '.registration.registrationId // empty')" != "" ] && break
  attempt=$((attempt + 1))
  sleep 0.1
done
[ "$attempt" -lt 20 ] || { printf 'registration-not-observed\n' >&2; exit 1; }
attempt=0
while [ "$attempt" -lt 20 ]; do
  status=$(curl -fsS -b "$director_cookie_jar" \
    "$director_url/api/v1/status/$session_id?surfaceSlot=reviewer-workbench")
  [ "$(printf '%s' "$status" | jq -r '.registration.registrationId // empty')" != "" ] && break
  attempt=$((attempt + 1))
  sleep 0.1
done
[ "$attempt" -lt 20 ] || { printf 'workbench-registration-not-observed\n' >&2; exit 1; }

stage='prepare-session'
body=$(jq -cn --argjson revision "$revision" \
  '{action:"prepare",expectedState:"preparing",expectedRevision:$revision,reason:"secure native assurance"}')
prepared=$(post_json "$director_cookie_jar" "$director_csrf" "$director_origin" \
  "$director_url/api/v1/sessions/$session_id/lifecycle" "$body")
revision=$(printf '%s' "$prepared" | jq -r '.session.revision')
stage='start-session'
body=$(jq -cn --argjson revision "$revision" \
  '{action:"start",expectedState:"ready",expectedRevision:$revision,reason:"secure native assurance"}')
started=$(post_json "$director_cookie_jar" "$director_csrf" "$director_origin" \
  "$director_url/api/v1/sessions/$session_id/lifecycle" "$body")
revision=$(printf '%s' "$started" | jq -r '.session.revision')

stage='arm-cue-delay'
body=$(jq -cn --argjson revision "$revision" \
  '{expectedRevision:$revision,delayMilliseconds:250}')
fault=$(post_json "$director_cookie_jar" "$director_csrf" "$director_origin" \
  "$director_url/api/v1/sessions/$session_id/cue-delay" "$body")

stage='synthetic-sign-in'
body='{"actorId":"synthetic-audience-user","surfaceSlot":"audience-display"}'
post_json "$director_cookie_jar" "$director_csrf" "$director_origin" \
  "$director_url/api/v1/sessions/$session_id/synthetic-sign-in" "$body" >/dev/null
attempt=0
while [ "$attempt" -lt 30 ]; do
  identity_context=$(curl -fsS -b "$gateway_cookie_jar" \
    "$gateway_url/api/v1/session-context")
  [ "$(printf '%s' "$identity_context" | jq -r '.syntheticStatus')" = "established" ] && break
  attempt=$((attempt + 1))
  sleep 0.1
done
[ "$attempt" -lt 30 ] || { printf 'synthetic-session-not-established\n' >&2; exit 1; }
[ "$(printf '%s' "$identity_context" | jq -r '.syntheticActorId')" = "synthetic-audience-user" ] || {
  printf 'synthetic-actor-mismatch\n' >&2
  exit 1
}

body='{"actorId":"synthetic-reviewer","surfaceSlot":"reviewer-workbench"}'
post_json "$director_cookie_jar" "$director_csrf" "$director_origin" \
  "$director_url/api/v1/sessions/$session_id/synthetic-sign-in" "$body" >/dev/null
attempt=0
while [ "$attempt" -lt 30 ]; do
  workbench_identity_context=$(curl -fsS -b "$workbench_cookie_jar" \
    "$gateway_url/api/v1/session-context")
  [ "$(printf '%s' "$workbench_identity_context" | jq -r '.syntheticStatus')" = "established" ] && break
  attempt=$((attempt + 1))
  sleep 0.1
done
[ "$attempt" -lt 30 ] || { printf 'workbench-synthetic-session-not-established\n' >&2; exit 1; }
[ "$(printf '%s' "$workbench_identity_context" | jq -r '.syntheticActorId')" = "synthetic-reviewer" ] || {
  printf 'workbench-synthetic-actor-mismatch\n' >&2
  exit 1
}
[ "$(printf '%s' "$workbench_identity_context" | jq -r '.syntheticRoles | index("workbench-reviewer") != null')" = true ] \
  && [ "$(printf '%s' "$workbench_identity_context" | jq -r '.environmentId | length > 8')" = true ] \
  && [ "$(printf '%s' "$workbench_identity_context" | jq -r '.trustProfile')" = environment-local-synthetic-root ] \
  && [ "$(printf '%s' "$workbench_identity_context" | jq -r '.maximumValidUntil | length > 10')" = true ] || {
  printf 'workbench-identity-banner-context-incomplete\n' >&2
  exit 1
}
if printf '%s' "$workbench_identity_context" | grep -Eq 'signature|grantId|sessionReference|PPL_APP_SESSION'; then
  printf 'protected-workbench-identity-material-disclosed\n' >&2
  exit 1
fi
if printf '%s' "$identity_context" | grep -Eq 'signature|grantId|sessionReference|PPL_APP_SESSION'; then
  printf 'protected-identity-material-disclosed\n' >&2
  exit 1
fi

stage='receive-cue'
curl -fsSN --max-time 5 -b "$gateway_cookie_jar" \
  "$gateway_url/api/v1/cues" >"$sse_capture" 2>/dev/null &
sse_pid=$!
sleep 0.2
body='{"surfaceSlot":"audience-display","semanticView":"pres-intro","heading":"A governed source, not a magic answer","message":"Semantic event path with synthetic information only.","expiresInSeconds":60}'
cue=$(post_json "$director_cookie_jar" "$director_csrf" "$director_origin" \
  "$director_url/api/v1/sessions/$session_id/cue" "$body")
wait "$sse_pid" || true
sse_cue=$(sed -n 's/^data: //p' "$sse_capture" | head -1)
[ "$(printf '%s' "$sse_cue" | jq -r '.cueId')" = "$(printf '%s' "$cue" | jq -r '.cueId')" ] || {
  printf 'sse-cue-mismatch\n' >&2
  exit 1
}

stage='record-outcome'
outcome=$(printf '%s' "$cue" | jq -c \
  '{contractId:"P-004",contractVersion:"1.0.0",outcomeId:("outcome:"+.cueId),cueId:.cueId,cueDigest:.cueDigest,sessionId:.sessionId,sessionRevision:.sessionRevision,surfaceSlot:.surfaceSlot,registrationId:.registrationId,registrationRevision:.registrationRevision,connectionGeneration:.connectionGeneration,semanticView:.semanticView,result:"applied",concludedAt:(now|todateiso8601),businessCompletionClaimed:false}')
post_json "$gateway_cookie_jar" "$gateway_csrf" "$gateway_origin" \
  "$gateway_url/api/v1/outcomes" "$outcome" >/dev/null

attempt=0
while [ "$attempt" -lt 20 ]; do
  status=$(curl -fsS -b "$director_cookie_jar" \
    "$director_url/api/v1/status/$session_id")
  [ "$(printf '%s' "$status" | jq -r '.presentationCheckpoint.result // empty')" = "satisfied" ] && break
  attempt=$((attempt + 1))
  sleep 0.1
done
[ "$attempt" -lt 20 ] || { printf 'checkpoint-not-satisfied\n' >&2; exit 1; }

stage='receive-workbench-views'
curl -fsSN --max-time 5 -b "$workbench_cookie_jar" \
  "$gateway_url/api/v1/cues" >"$workbench_sse_capture" 2>/dev/null &
workbench_sse_pid=$!
sleep 0.2
body='{"surfaceSlot":"reviewer-workbench","semanticView":"wb-engagement","heading":"Harbour support policy review","message":"Open the bounded synthetic engagement context.","expiresInSeconds":60}'
engagement_cue=$(post_json "$director_cookie_jar" "$director_csrf" "$director_origin" \
  "$director_url/api/v1/sessions/$session_id/cue" "$body")
body='{"surfaceSlot":"reviewer-workbench","semanticView":"wb-source-intake","heading":"Add a source for governed review","message":"Show human-operated intake controls without submitting anything.","expiresInSeconds":60}'
intake_cue=$(post_json "$director_cookie_jar" "$director_csrf" "$director_origin" \
  "$director_url/api/v1/sessions/$session_id/cue" "$body")
wait "$workbench_sse_pid" || true
engagement_sse_cue=$(sed -n 's/^data: //p' "$workbench_sse_capture" \
  | jq -sc 'map(select(.semanticView == "wb-engagement"))[0]')
intake_sse_cue=$(sed -n 's/^data: //p' "$workbench_sse_capture" \
  | jq -sc 'map(select(.semanticView == "wb-source-intake"))[0]')
rm -f "$workbench_sse_capture"
[ "$(printf '%s' "$engagement_sse_cue" | jq -r '.cueId')" = "$(printf '%s' "$engagement_cue" | jq -r '.cueId')" ] || {
  printf 'workbench-engagement-cue-mismatch\n' >&2
  exit 1
}
[ "$(printf '%s' "$intake_sse_cue" | jq -r '.cueId')" = "$(printf '%s' "$intake_cue" | jq -r '.cueId')" ] || {
  printf 'workbench-intake-cue-mismatch\n' >&2
  exit 1
}
for applied_cue in "$engagement_cue" "$intake_cue"; do
  outcome=$(printf '%s' "$applied_cue" | jq -c \
    '{contractId:"P-004",contractVersion:"1.0.0",outcomeId:("outcome:"+.cueId),cueId:.cueId,cueDigest:.cueDigest,sessionId:.sessionId,sessionRevision:.sessionRevision,surfaceSlot:.surfaceSlot,registrationId:.registrationId,registrationRevision:.registrationRevision,connectionGeneration:.connectionGeneration,semanticView:.semanticView,result:"applied",concludedAt:(now|todateiso8601),businessCompletionClaimed:false}')
  post_json "$workbench_cookie_jar" "$workbench_csrf" "$gateway_origin" \
    "$gateway_url/api/v1/outcomes" "$outcome" >/dev/null
done

stage='unsupported-view-refusal'
body='{"surfaceSlot":"reviewer-workbench","semanticView":"wb-not-admitted","heading":"Unsupported view test","message":"This must be refused before delivery.","expiresInSeconds":60}'
unsupported_status=$(curl -sS -o /dev/null -w '%{http_code}' -b "$director_cookie_jar" \
  -H "Origin: $director_origin" -H "X-PPL-CSRF: $director_csrf" \
  -H 'Content-Type: application/json' -X POST \
  "$director_url/api/v1/sessions/$session_id/cue" -d "$body")
[ "$unsupported_status" = 409 ] || {
  printf 'unsupported-view-not-refused\n' >&2
  exit 1
}

stage='pause-session'
body=$(jq -cn --argjson revision "$revision" \
  '{action:"pause",expectedState:"running",expectedRevision:$revision,reason:"Gate B pause assurance"}')
paused=$(post_json "$director_cookie_jar" "$director_csrf" "$director_origin" \
  "$director_url/api/v1/sessions/$session_id/lifecycle" "$body")
revision=$(printf '%s' "$paused" | jq -r '.session.revision')
body=$(jq -cn --argjson revision "$revision" \
  '{action:"resume",expectedState:"paused",expectedRevision:$revision,reason:"Gate B resume assurance"}')
resumed=$(post_json "$director_cookie_jar" "$director_csrf" "$director_origin" \
  "$director_url/api/v1/sessions/$session_id/lifecycle" "$body")
revision=$(printf '%s' "$resumed" | jq -r '.session.revision')

stage='stop-session'
body=$(jq -cn --argjson revision "$revision" \
  '{action:"stop",expectedState:"running",expectedRevision:$revision,reason:"secure native assurance"}')
stopped=$(post_json "$director_cookie_jar" "$director_csrf" "$director_origin" \
  "$director_url/api/v1/sessions/$session_id/lifecycle" "$body")
revision=$(printf '%s' "$stopped" | jq -r '.session.revision')
stage='reset-session'
body=$(jq -cn --argjson revision "$revision" \
  '{action:"reset",expectedState:"stopped",expectedRevision:$revision,reason:"secure native assurance"}')
reset=$(post_json "$director_cookie_jar" "$director_csrf" "$director_origin" \
  "$director_url/api/v1/sessions/$session_id/lifecycle" "$body")
successor=$(printf '%s' "$reset" | jq -r '.successor.sessionId')
successor_status=$(curl -fsS -b "$director_cookie_jar" \
  "$director_url/api/v1/status/$successor")

stage='complete'
attempt=0
while [ "$attempt" -lt 30 ]; do
  terminated_context=$(curl -fsS -b "$gateway_cookie_jar" \
    "$gateway_url/api/v1/session-context")
  [ "$(printf '%s' "$terminated_context" | jq -r '.syntheticStatus')" = "not-established" ] && break
  attempt=$((attempt + 1))
  sleep 0.1
done
[ "$attempt" -lt 30 ] || {
  printf 'synthetic-session-not-terminated\n' >&2
  exit 1
}
attempt=0
while [ "$attempt" -lt 30 ]; do
  terminated_workbench_context=$(curl -fsS -b "$workbench_cookie_jar" \
    "$gateway_url/api/v1/session-context")
  [ "$(printf '%s' "$terminated_workbench_context" | jq -r '.syntheticStatus')" = "not-established" ] && break
  attempt=$((attempt + 1))
  sleep 0.1
done
[ "$attempt" -lt 30 ] || {
  printf 'workbench-synthetic-session-not-terminated\n' >&2
  exit 1
}

printf 'session=%s\ncsrf_refusal=%s\nsynthetic_actor=%s\nworkbench_actor=%s\nsecure_fault=%s\nsse_view=%s\nworkbench_views=%s,%s\nunsupported_view_status=%s\npause_state=%s\ncheckpoint=%s\nprior_state=%s\nsuccessor=%s\nsuccessor_checkpoint=%s\nsuccessor_logical_time_initialised=%s\n' \
  "$session_id" \
  "$csrf_status" \
  "$(printf '%s' "$identity_context" | jq -r '.syntheticActorId')" \
  "$(printf '%s' "$workbench_identity_context" | jq -r '.syntheticActorId')" \
  "$(printf '%s' "$fault" | jq -r '.code')" \
  "$(printf '%s' "$sse_cue" | jq -r '.semanticView')" \
  "$(printf '%s' "$engagement_sse_cue" | jq -r '.semanticView')" \
  "$(printf '%s' "$intake_sse_cue" | jq -r '.semanticView')" \
  "$unsupported_status" \
  "$(printf '%s' "$paused" | jq -r '.session.state')" \
  "$(printf '%s' "$status" | jq -r '.presentationCheckpoint.result')" \
  "$(printf '%s' "$reset" | jq -r '.session.state')" \
  "$successor" \
  "$(printf '%s' "$successor_status" | jq -r '.presentationCheckpoint // "none"')" \
  "$(printf '%s' "$successor_status" | jq -r '.session.logicalTimeInitialised')"

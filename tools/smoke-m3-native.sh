#!/bin/sh
set -eu

stage=initialisation
trap 'result=$?; if [ "$result" -ne 0 ]; then printf "smoke-failed-at=%s\n" "$stage" >&2; fi' EXIT

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

login_cookie() {
  origin=$1
  url=$2
  curl -fsS -D - -o /dev/null -H "Origin: $origin" \
    -H 'Content-Type: application/json' -X POST \
    "$url/api/v1/development-session" -d '{}' |
    awk -F '[=;]' 'tolower($1) ~ /^set-cookie: ppl_dev_session$/ { print "PPL_DEV_SESSION=" $2; exit }'
}

post_json() {
  cookie=$1
  origin=$2
  url=$3
  body=$4
  curl -sS --fail-with-body -H "Cookie: $cookie" -H "Origin: $origin" \
    -H 'Content-Type: application/json' -X POST "$url" -d "$body"
}

require_command curl
require_command jq
director_cookie=$(login_cookie "$director_origin" "$director_url")
gateway_cookie=$(login_cookie "$gateway_origin" "$gateway_url")
[ -n "$director_cookie" ] && [ -n "$gateway_cookie" ] || {
  printf 'development-session-cookie-unavailable\n' >&2
  exit 1
}

stage=create-session
created=$(post_json "$director_cookie" "$director_origin" \
  "$director_url/api/v1/sessions" '{}')
session_id=$(printf '%s' "$created" | jq -r '.session.sessionId')
revision=$(printf '%s' "$created" | jq -r '.session.revision')

stage=set-logical-time
body=$(jq -cn --argjson revision "$revision" \
  '{operation:"set",expectedRevision:$revision,logicalInstant:"2030-01-01T09:00:00Z"}')
logical_time=$(post_json "$director_cookie" "$director_origin" \
  "$director_url/api/v1/sessions/$session_id/logical-time" "$body")
revision=$(printf '%s' "$logical_time" | jq -r '.sessionRevision')

stage=register-surface
body=$(jq -cn --arg session "$session_id" \
  '{sessionId:$session,surfaceSlot:"audience-display",surfaceRole:"audience-display"}')
post_json "$gateway_cookie" "$gateway_origin" \
  "$gateway_url/api/v1/registrations" "$body" >/dev/null

attempt=0
while [ "$attempt" -lt 20 ]; do
  status=$(curl -fsS -H "Cookie: $director_cookie" \
    "$director_url/api/v1/status/$session_id")
  [ "$(printf '%s' "$status" | jq -r '.registration.registrationId // empty')" != "" ] && break
  attempt=$((attempt + 1))
  sleep 0.1
done
[ "$attempt" -lt 20 ] || { printf 'registration-not-observed\n' >&2; exit 1; }

stage=prepare-session
body=$(jq -cn --argjson revision "$revision" \
  '{action:"prepare",expectedState:"preparing",expectedRevision:$revision,reason:"secure native assurance"}')
prepared=$(post_json "$director_cookie" "$director_origin" \
  "$director_url/api/v1/sessions/$session_id/lifecycle" "$body")
revision=$(printf '%s' "$prepared" | jq -r '.session.revision')
stage=start-session
body=$(jq -cn --argjson revision "$revision" \
  '{action:"start",expectedState:"ready",expectedRevision:$revision,reason:"secure native assurance"}')
started=$(post_json "$director_cookie" "$director_origin" \
  "$director_url/api/v1/sessions/$session_id/lifecycle" "$body")
revision=$(printf '%s' "$started" | jq -r '.session.revision')

stage=arm-cue-delay
body=$(jq -cn --argjson revision "$revision" \
  '{expectedRevision:$revision,delayMilliseconds:250}')
fault=$(post_json "$director_cookie" "$director_origin" \
  "$director_url/api/v1/sessions/$session_id/cue-delay" "$body")

stage=receive-cue
curl -fsSN --max-time 5 -H "Cookie: $gateway_cookie" \
  "$gateway_url/api/v1/cues" >"$sse_capture" 2>/dev/null &
sse_pid=$!
sleep 0.2
body='{"surfaceSlot":"audience-display","semanticView":"assurance-welcome","heading":"Secure native assurance","message":"Semantic event path with synthetic information only.","expiresInSeconds":60}'
cue=$(post_json "$director_cookie" "$director_origin" \
  "$director_url/api/v1/sessions/$session_id/cue" "$body")
wait "$sse_pid" || true
sse_cue=$(sed -n 's/^data: //p' "$sse_capture" | head -1)
[ "$(printf '%s' "$sse_cue" | jq -r '.cueId')" = "$(printf '%s' "$cue" | jq -r '.cueId')" ] || {
  printf 'sse-cue-mismatch\n' >&2
  exit 1
}

stage=record-outcome
outcome=$(printf '%s' "$cue" | jq -c \
  '{contractId:"P-004",contractVersion:"1.0.0",outcomeId:("outcome:"+.cueId),cueId:.cueId,cueDigest:.cueDigest,sessionId:.sessionId,sessionRevision:.sessionRevision,surfaceSlot:.surfaceSlot,registrationId:.registrationId,registrationRevision:.registrationRevision,connectionGeneration:.connectionGeneration,semanticView:.semanticView,result:"applied",concludedAt:(now|todateiso8601),businessCompletionClaimed:false}')
post_json "$gateway_cookie" "$gateway_origin" \
  "$gateway_url/api/v1/outcomes" "$outcome" >/dev/null

attempt=0
while [ "$attempt" -lt 20 ]; do
  status=$(curl -fsS -H "Cookie: $director_cookie" \
    "$director_url/api/v1/status/$session_id")
  [ "$(printf '%s' "$status" | jq -r '.presentationCheckpoint.result // empty')" = "satisfied" ] && break
  attempt=$((attempt + 1))
  sleep 0.1
done
[ "$attempt" -lt 20 ] || { printf 'checkpoint-not-satisfied\n' >&2; exit 1; }

stage=stop-session
body=$(jq -cn --argjson revision "$revision" \
  '{action:"stop",expectedState:"running",expectedRevision:$revision,reason:"secure native assurance"}')
stopped=$(post_json "$director_cookie" "$director_origin" \
  "$director_url/api/v1/sessions/$session_id/lifecycle" "$body")
revision=$(printf '%s' "$stopped" | jq -r '.session.revision')
stage=reset-session
body=$(jq -cn --argjson revision "$revision" \
  '{action:"reset",expectedState:"stopped",expectedRevision:$revision,reason:"secure native assurance"}')
reset=$(post_json "$director_cookie" "$director_origin" \
  "$director_url/api/v1/sessions/$session_id/lifecycle" "$body")
successor=$(printf '%s' "$reset" | jq -r '.successor.sessionId')
successor_status=$(curl -fsS -H "Cookie: $director_cookie" \
  "$director_url/api/v1/status/$successor")

stage=complete
printf 'session=%s\nsecure_fault=%s\nsse_view=%s\ncheckpoint=%s\nprior_state=%s\nsuccessor=%s\nsuccessor_checkpoint=%s\nsuccessor_logical_time_initialised=%s\n' \
  "$session_id" \
  "$(printf '%s' "$fault" | jq -r '.code')" \
  "$(printf '%s' "$sse_cue" | jq -r '.semanticView')" \
  "$(printf '%s' "$status" | jq -r '.presentationCheckpoint.result')" \
  "$(printf '%s' "$reset" | jq -r '.session.state')" \
  "$successor" \
  "$(printf '%s' "$successor_status" | jq -r '.presentationCheckpoint // "none"')" \
  "$(printf '%s' "$successor_status" | jq -r '.session.logicalTimeInitialised')"

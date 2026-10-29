#!/usr/bin/env bash
# Write an observation after reset to the debug log (to be run by the user)

set -euo pipefail

SESSION="55c9bc"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LOG="$ROOT/logs/debug-host.ndjson"
OBS="${1:-unknown}"
TS=$(($(date +%s) * 1000))

mkdir -p "$(dirname "$LOG")"
printf '{"sessionId":"%s","timestamp":%s,"hypothesisId":"VIS","location":"debug-observe.sh","message":"user_observation","data":{"observation":"%s"},"runId":"user-report"}\n' \
    "$SESSION" "$TS" "$OBS" >> "$LOG"

echo "→ Observation logged: $OBS"

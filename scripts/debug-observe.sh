#!/usr/bin/env bash
# Beobachtung nach Reset ins Debug-Log schreiben (vom User auszuführen)

set -euo pipefail

LOG="/Users/marcel/Development/mk/embedded/adafruit/.cursor/debug-55c9bc.log"
SESSION="55c9bc"
OBS="${1:-unknown}"
TS=$(($(date +%s) * 1000))

mkdir -p "$(dirname "$LOG")"
printf '{"sessionId":"%s","timestamp":%s,"hypothesisId":"VIS","location":"debug-observe.sh","message":"user_observation","data":{"observation":"%s"},"runId":"user-report"}\n' \
    "$SESSION" "$TS" "$OBS" >> "$LOG"

echo "→ Beobachtung geloggt: $OBS"

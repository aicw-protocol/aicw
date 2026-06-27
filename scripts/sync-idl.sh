#!/usr/bin/env bash
# Copy built IDL to aicw_app (and optionally predict dashboard).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/target/idl/aicw.json"

if [[ ! -f "$SRC" ]]; then
  echo "Missing $SRC — run anchor build first."
  exit 1
fi

APP_IDL="$ROOT/../aicw_app/src/idl/aicw.json"
if [[ -d "$(dirname "$APP_IDL")" ]]; then
  cp "$SRC" "$APP_IDL"
  echo "Synced -> $APP_IDL"
else
  echo "Skip: aicw_app not found at $APP_IDL"
fi

PREDICT_IDL="$ROOT/../predict/app/dashboard/src/idl/aicw.json"
if [[ -d "$(dirname "$PREDICT_IDL")" ]]; then
  cp "$SRC" "$PREDICT_IDL"
  echo "Synced -> $PREDICT_IDL"
fi

echo "Done. Commit IDL copies and redeploy apps."

#!/usr/bin/env bash
# Pre-flight checks before AICW mainnet deploy. Run from repo root in WSL:
#   bash scripts/preflight-mainnet.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

PROGRAM_SO="$ROOT/target/deploy/aicw.so"
KEYPAIR="$ROOT/target/deploy/aicw-keypair.json"
MIN_SOL="${MIN_DEPLOY_SOL:-3}"
RPC="${SOLANA_RPC_URL:-https://api.mainnet-beta.solana.com}"

echo "=== AICW mainnet preflight ==="
echo "Repo: $ROOT"
echo "RPC:  $RPC"
echo

fail=0
warn=0

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "FAIL: missing command: $1"
    fail=1
  fi
}

require_cmd solana
require_cmd anchor

if [[ ! -f "$PROGRAM_SO" ]]; then
  echo "FAIL: $PROGRAM_SO not found. Run: anchor build"
  fail=1
else
  bytes=$(wc -c < "$PROGRAM_SO" | tr -d ' ')
  rent=$(solana rent "$bytes" 2>/dev/null | awk '{print $NF}')
  echo "OK:   program binary $bytes bytes (rent-exempt ~ $rent SOL)"
fi

if [[ ! -f "$KEYPAIR" ]]; then
  echo "FAIL: $KEYPAIR not found. anchor build creates this; back up before deploy."
  fail=1
else
  program_id=$(solana address -k "$KEYPAIR")
  echo "OK:   program keypair -> $program_id"
  if grep -q 'declare_id!("'"$program_id"'")' "$ROOT/programs/aicw/src/lib.rs"; then
    echo "OK:   declare_id! matches keypair"
  else
    echo "FAIL: declare_id! in lib.rs does not match keypair $program_id"
    fail=1
  fi
fi

deployer=$(solana address 2>/dev/null || true)
if [[ -z "$deployer" ]]; then
  echo "FAIL: solana CLI wallet not configured"
  fail=1
else
  echo "OK:   deployer wallet -> $deployer"
  balance=$(solana balance "$deployer" --url "$RPC" 2>/dev/null | awk '{print $1}' || echo "0")
  echo "      mainnet balance: $balance SOL (need >= $MIN_SOL SOL recommended)"
  if awk -v b="$balance" -v m="$MIN_SOL" 'BEGIN { exit (b+0 >= m+0) ? 0 : 1 }'; then
    echo "OK:   balance meets MIN_DEPLOY_SOL=$MIN_SOL"
  else
    echo "FAIL: insufficient mainnet SOL on deployer wallet"
    fail=1
  fi
fi

if solana program show "$program_id" --url "$RPC" >/dev/null 2>&1; then
  echo "WARN: program $program_id already exists on mainnet"
  warn=1
  solana program show "$program_id" --url "$RPC" || true
else
  echo "OK:   program not yet deployed on mainnet (fresh deploy)"
fi

if [[ ! -f "$ROOT/target/idl/aicw.json" ]]; then
  echo "WARN: target/idl/aicw.json missing — run anchor build"
  warn=1
else
  echo "OK:   IDL present at target/idl/aicw.json"
fi

if ! grep -q '\[programs.mainnet\]' "$ROOT/Anchor.toml"; then
  echo "WARN: Anchor.toml has no [programs.mainnet] section"
  warn=1
else
  echo "OK:   Anchor.toml includes [programs.mainnet]"
fi

echo
if [[ "$fail" -gt 0 ]]; then
  echo "Preflight FAILED ($fail blocking issue(s), $warn warning(s))."
  exit 1
fi

if [[ "$warn" -gt 0 ]]; then
  echo "Preflight passed with $warn warning(s). Review before deploy."
  exit 0
fi

echo "Preflight PASSED. Safe to proceed with scripts/deploy-mainnet.sh"

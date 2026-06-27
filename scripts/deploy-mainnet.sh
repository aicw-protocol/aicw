#!/usr/bin/env bash
# Deploy AICW to Solana mainnet-beta (upgradeable). Run in WSL from repo root:
#   bash scripts/preflight-mainnet.sh
#   bash scripts/deploy-mainnet.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

RPC="${SOLANA_RPC_URL:-https://api.mainnet-beta.solana.com}"
KEYPAIR="$ROOT/target/deploy/aicw-keypair.json"
PROGRAM_ID="$(solana address -k "$KEYPAIR")"

echo "=== AICW mainnet deploy ==="
echo "Program ID: $PROGRAM_ID"
echo "RPC:        $RPC"
echo

bash "$ROOT/scripts/preflight-mainnet.sh"

solana config set --url "$RPC" >/dev/null
echo "Building..."
anchor build

echo "Deploying to mainnet-beta..."
anchor deploy --provider.cluster mainnet

echo
echo "Program status:"
solana program show "$PROGRAM_ID" --url "$RPC"

echo
echo "Next steps:"
echo "  1. bash scripts/sync-idl.sh"
echo "  2. Secure the deploy keypair and upgrade authority wallet (offline backup)."
echo "  3. Verify: solana program show $PROGRAM_ID --url $RPC"
echo "  4. Update app env: NEXT_PUBLIC_SOLANA_CLUSTER=mainnet-beta, RPC, MPC bridge"
echo "  5. Do NOT run: solana program set-upgrade-authority $PROGRAM_ID --final"
echo "  6. See MAINNET_DEPLOY.md post-deploy checklist"

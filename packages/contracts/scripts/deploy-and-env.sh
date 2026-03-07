#!/usr/bin/env bash
set -euo pipefail

NETWORK="${1:?Usage: deploy-and-env.sh <anvil|sepolia|fuji>}"
CONTRACTS_DIR="$(cd "$(dirname "$0")/.." && pwd)"
WEBAPP_ENV="$CONTRACTS_DIR/../../apps/webapp/.env"

# ── Network config ──────────────────────────────────────────────
case "$NETWORK" in
  anvil)
    CHAIN_ID=31337
    RPC_URL="http://127.0.0.1:8545"
    FORGE_ARGS="--rpc-url $RPC_URL --broadcast --unlocked --sender 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 --code-size-limit 50000"
    ;;
  sepolia)
    CHAIN_ID=11155111
    RPC_URL="${SEPOLIA_RPC:?SEPOLIA_RPC not set}"
    FORGE_ARGS="--rpc-url sepolia --account ${SEPOLIA_DEPLOYER_NAME:?SEPOLIA_DEPLOYER_NAME not set} --broadcast --verify -vvvv"
    ;;
  fuji)
    CHAIN_ID=43113
    RPC_URL="${FUJI_RPC:?FUJI_RPC not set}"
    FORGE_ARGS="--rpc-url $RPC_URL --account ${FUJI_DEPLOYER_NAME:?FUJI_DEPLOYER_NAME not set} --broadcast -vvvv"
    ;;
  *)
    echo "Unknown network: $NETWORK"
    echo "Usage: deploy-and-env.sh <anvil|sepolia|fuji>"
    exit 1
    ;;
esac

# ── Deploy ──────────────────────────────────────────────────────
echo ">> Deploying to $NETWORK (chainId=$CHAIN_ID)..."
cd "$CONTRACTS_DIR"
forge script Deploy $FORGE_ARGS

# ── Parse broadcast JSON ────────────────────────────────────────
BROADCAST="$CONTRACTS_DIR/broadcast/Deploy.sol/$CHAIN_ID/run-latest.json"

if [[ ! -f "$BROADCAST" ]]; then
  echo "ERROR: Broadcast file not found at $BROADCAST"
  exit 1
fi

VERIFIER=$(jq -r '[.transactions[] | select(.contractName=="HonkVerifier")][0].contractAddress' "$BROADCAST")
OGBANK=$(jq -r '[.transactions[] | select(.contractName=="OGBankContract")][0].contractAddress' "$BROADCAST")

# ── Token addresses ─────────────────────────────────────────────
case "$NETWORK" in
  anvil)
    # Mocks were deployed — extract from broadcast by deploy order
    COLLATERAL=$(jq -r '[.transactions[] | select(.contractName=="MockERC20")][0].contractAddress' "$BROADCAST")
    BORROW=$(jq -r '[.transactions[] | select(.contractName=="MockERC20")][1].contractAddress' "$BROADCAST")
    AAVE_POOL=$(jq -r '[.transactions[] | select(.contractName=="MockAavePool")][0].contractAddress' "$BROADCAST")
    ;;
  sepolia)
    COLLATERAL=""
    BORROW=""
    AAVE_POOL=""
    ;;
  fuji)
    COLLATERAL="0xd00ae08403B9bbb9124bB305C09058E32C39A48c"
    BORROW="0x5425890298aed601595a70AB815c96711a31Bc65"
    AAVE_POOL="0x794a61358D6845594F94dc1DB02A252b5b4814aD"
    ;;
esac

# ── Write .env ──────────────────────────────────────────────────
cat > "$WEBAPP_ENV" <<EOF
VITE_OGBANK_ADDRESS=$OGBANK
VITE_COLLATERAL_TOKEN_ADDRESS=$COLLATERAL
VITE_BORROW_TOKEN_ADDRESS=$BORROW
VITE_AAVE_POOL_ADDRESS=$AAVE_POOL
VITE_VERIFIER_ADDRESS=$VERIFIER
VITE_CHAIN_ID=$CHAIN_ID
VITE_RPC_URL=$RPC_URL
VITE_WALLETCONNECT_PROJECT_ID=${VITE_WALLETCONNECT_PROJECT_ID:-}
EOF

echo ""
echo ">> Deployed:"
echo "   Verifier  = $VERIFIER"
echo "   OGBank    = $OGBANK"
echo "   Collateral= $COLLATERAL"
echo "   Borrow    = $BORROW"
echo "   AavePool  = $AAVE_POOL"
echo ""
echo ">> .env written to $WEBAPP_ENV"

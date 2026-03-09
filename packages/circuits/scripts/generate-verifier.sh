#!/usr/bin/env bash
set -euo pipefail

CIRCUITS_DIR="$(cd "$(dirname "$0")/.." && pwd)"
CONTRACTS_DIR="$(cd "$CIRCUITS_DIR/../contracts" && pwd)"
ARTIFACT="$CIRCUITS_DIR/target/circuits.json"
VK="$CIRCUITS_DIR/target/vk"
VERIFIER_OUT="$CIRCUITS_DIR/target/Verifier.sol"
VERIFIER_DEST="$CONTRACTS_DIR/src/contracts/Verifier.sol"

echo ">> Compiling Noir circuit..."
cd "$CIRCUITS_DIR"
nargo compile

echo ">> Generating verification key (keccak)..."
bb write_vk -b "$ARTIFACT" -o "$CIRCUITS_DIR/target" --oracle_hash keccak

echo ">> Generating Solidity verifier..."
bb write_solidity_verifier -k "$VK" -o "$VERIFIER_OUT"

echo ">> Copying Verifier.sol to contracts..."
cp "$VERIFIER_OUT" "$VERIFIER_DEST"

echo ">> Building contracts..."
cd "$CONTRACTS_DIR"
forge build

echo ">> Done. Verifier.sol at $VERIFIER_DEST"

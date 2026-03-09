# OGBank

Private lending protocol powered by ZK proofs on Avalanche. Users supply ZEC collateral, borrow USDC via Aave V3, and prove repayment with zero-knowledge proofs to withdraw collateral — all without linking deposit and withdrawal addresses.

## Architecture

```
packages/
  circuits/     Noir ZK circuit (borrow + auth modes)
  contracts/    Solidity contracts (OGBankContract, Verifier, mocks)
apps/
  landing/      Marketing landing page
  webapp/       Protocol dashboard (connect wallet, view balances)
```

## Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Node.js | >= 18 | https://nodejs.org |
| Foundry (forge, anvil) | latest | `curl -L https://foundry.paradigm.xyz \| bash && foundryup` |
| Nargo (Noir) | 1.0.0-beta.19 | https://noir-lang.org/docs/getting_started |
| bb (Barretenberg) | latest | Installed with Nargo |
| jq | any | `brew install jq` |

## Setup

```bash
# Install all workspace dependencies
npm install

# Install Foundry dependencies
cd packages/contracts && forge install && cd ../..
```

## Local Development (anvil)

### 1. Start anvil

The HonkVerifier contract exceeds the default EVM contract size limit, so anvil must be started with `--code-size-limit`:

```bash
npm run anvil:start
```

This runs `anvil --code-size-limit 50000` on `http://127.0.0.1:8545`.

### 2. Deploy contracts

In a separate terminal:

```bash
npm run contracts:deploy:anvil
```

This does three things:
1. Deploys HonkVerifier, MockERC20 (ZEC + USDC), MockAavePool, and OGBankContract to anvil
2. Mints 10,000 ZEC and 10,000 USDC to the default anvil account (`0xf39F...2266`)
3. Writes all contract addresses to `apps/webapp/.env`

### 3. Start the webapp

```bash
npm run webapp:dev
```

Open http://localhost:5173, connect with MetaMask (import anvil's first private key: `0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80`), and you should see your native ETH balance plus 10,000 ZEC (collateral) and 10,000 USDC (borrow).

## ZK Circuits

```bash
# Compile the Noir circuit
npm run circuits:build

# Run circuit tests
npm run circuits:test

# Regenerate the Solidity verifier from the compiled circuit
npm run circuits:verifier
```

After regenerating the verifier, rebuild contracts with `npm run contracts:build`.

## Smart Contracts

```bash
# Build
npm run contracts:build

# Run all tests
npm run contracts:test

# Unit tests only
npm run contracts:test:unit

# Integration tests only
npm run contracts:test:integration

# Coverage
npm run contracts:coverage
```

## Testnet Deployment

### Avalanche Fuji

Create `packages/contracts/.env`:

```
FUJI_RPC=https://api.avax-test.network/ext/bc/C/rpc
FUJI_DEPLOYER_NAME=my-fuji-deployer
```

Then:

```bash
cd packages/contracts
npm run deploy:fuji
```

### Sepolia

Create `packages/contracts/.env`:

```
SEPOLIA_RPC=https://eth-sepolia.g.alchemy.com/v2/YOUR_KEY
SEPOLIA_DEPLOYER_NAME=my-sepolia-deployer
```

Then:

```bash
cd packages/contracts
npm run deploy:sepolia
```

## All Root Scripts

| Script | Description |
|--------|-------------|
| `npm run anvil:start` | Start local anvil node |
| `npm run contracts:deploy:anvil` | Deploy to anvil + write webapp .env |
| `npm run contracts:build` | Compile Solidity contracts |
| `npm run contracts:test` | Run all Forge tests |
| `npm run contracts:test:unit` | Run unit tests |
| `npm run contracts:test:integration` | Run integration tests |
| `npm run contracts:coverage` | Generate coverage report |
| `npm run circuits:build` | Compile Noir circuit |
| `npm run circuits:test` | Run circuit tests |
| `npm run circuits:verifier` | Regenerate Solidity verifier |
| `npm run webapp:dev` | Start webapp dev server |
| `npm run webapp:build` | Build webapp for production |
| `npm run landing:dev` | Start landing page dev server |
| `npm run landing:build` | Build landing page |

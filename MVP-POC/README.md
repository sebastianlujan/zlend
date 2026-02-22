# ZLend MVP — Privacy-Preserving Cross-Chain Lending

Deposit shielded ZEC as collateral, borrow ERC-20 tokens on Avalanche.

```
User Wallet ──(shielded ZEC)──> Zcash Network
                                     │
zlend-cli ──(REST)──> zlend-relayer ──┤── trial decrypt (ivk)
                           │          │── verify deposit
                           │          └── update balance
                           │
                           └──(EVM tx)──> ZLendMVP.sol (Avalanche Fuji)
                                              └── transfer mUSDC to borrower
```

## Architecture

Three Rust crates + Solidity contracts:

| Crate | Type | Purpose |
|-------|------|---------|
| `zlend-core` | Library | Zcash key derivation (ZIP-32), Orchard trial decryption, SK encryption (ChaCha20-Poly1305 + Argon2) |
| `zlend-cli` | Binary | User-facing CLI — generate identity, register, check balance, scan, borrow |
| `zlend-relayer` | Binary | REST API server — holds encrypted keys, scans Zcash txs, signs EVM transactions |

| Contract | Purpose |
|----------|---------|
| `ZLendMVP.sol` | Trusted-relayer borrow with 150% collateral ratio |
| `MockUSDC.sol` | Test ERC-20 token (6 decimals) |

## Prerequisites

- **Rust** toolchain (`rustup` — https://rustup.rs)
- **Foundry** for Solidity tests (`curl -L https://foundry.paradigm.xyz | bash` then `foundryup`)
- **curl**, **jq**, **bc** for the e2e shell script

## Quick Start

```bash
cd MVP-POC

# Build everything
cargo build

# Run all 34 Rust tests
cargo test

# Run Solidity tests
cd contracts && forge test && cd ..

# Run the full end-to-end demo
chmod +x scripts/e2e.sh
./scripts/e2e.sh
```

## Project Structure

```
MVP-POC/
├── Cargo.toml                          # Workspace root
├── scripts/
│   └── e2e.sh                          # Full flow demo script
├── crates/
│   ├── zlend-core/                     # Shared crypto library
│   │   ├── src/
│   │   │   ├── lib.rs                  # Module exports
│   │   │   ├── keys.rs                 # ZIP-32 key derivation from BIP-39 mnemonic
│   │   │   ├── crypto.rs              # SK encryption at rest (Argon2 + ChaCha20-Poly1305)
│   │   │   └── scan.rs                # Orchard trial decryption (compact note decryption)
│   │   └── tests/
│   │       └── e2e_zcash_flow.rs      # 10 tests: keygen, encryption, trial decryption
│   ├── zlend-cli/                      # User CLI tool
│   │   └── src/main.rs                # 5 commands: generate, register, balance, scan, borrow
│   └── zlend-relayer/                  # REST API server
│       ├── src/
│       │   ├── main.rs                # Server startup, env var loading
│       │   ├── lib.rs                 # Library exports for integration tests
│       │   ├── api.rs                 # 6 HTTP endpoints + AppState
│       │   ├── db.rs                  # SQLite schema (positions, notes, loans)
│       │   ├── zcash.rs              # Tatum REST API client
│       │   └── evm.rs                # Alloy EVM signer for Avalanche contract calls
│       └── tests/
│           └── e2e_api_flow.rs        # 6 tests: full lifecycle, collateral, double-borrow
└── contracts/                          # Foundry project
    ├── src/
    │   ├── ZLendMVP.sol               # Core contract — relayer attestation borrow
    │   └── MockUSDC.sol               # Test ERC-20 (6 decimals)
    ├── test/
    │   └── ZLendMVP.t.sol             # 13 Foundry tests
    └── script/
        └── Deploy.s.sol               # Fuji testnet deployment
```

## Step-by-Step Lending Flow

### Phase 1: Generate Identity

```bash
cargo run -p zlend-cli -- generate
```

What happens:
1. Generates a 24-word BIP-39 mnemonic
2. Derives Orchard keys via ZIP-32 at path `m/32'/133'/0'`
3. Produces: spending key (sk), full viewing key (fvk), incoming viewing key (ivk), shielded address
4. Saves keys to `~/.zlend/keys.json`

### Phase 2: Start Relayer & Register

```bash
# Terminal 1: start the relayer
RELAYER_SK_PASSPHRASE="my-secret" cargo run -p zlend-relayer

# Terminal 2: register with the relayer
cargo run -p zlend-cli -- register --relayer http://localhost:3000
```

What happens:
1. CLI sends sk, fvk, ivk, address to the relayer
2. Relayer encrypts the spending key using ChaCha20-Poly1305 (key derived from passphrase via Argon2id)
3. Stores encrypted sk + plaintext fvk/ivk in SQLite
4. Returns a position UUID
5. CLI saves the position ID locally for future commands

### Phase 3: Deposit ZEC

Send shielded ZEC to the generated address using any Zcash wallet (zecwallet, ywallet). Note the transaction ID.

### Phase 4: Scan Transaction

```bash
cargo run -p zlend-cli -- scan --relayer http://localhost:3000 --txid <txid>
```

What happens:
1. Relayer fetches the transaction from Tatum API
2. Extracts Orchard actions (encrypted outputs)
3. Performs trial decryption using the stored IVK
4. If decryption succeeds: records the note value, updates position balance
5. Returns: found/not-found + value in zatoshis

> Without a Tatum API key, the relayer falls back to a simulated 1.5 ZEC deposit for development.

### Phase 5: Borrow Against Collateral

```bash
cargo run -p zlend-cli -- borrow --relayer http://localhost:3000 --amount 100 --recipient 0xYourAddress
```

What happens:
1. Relayer verifies collateral ratio: `balance_zat * 100 >= borrow_amount * 150` (150% ratio)
2. If EVM signer is configured: submits `borrow()` call to ZLendMVP contract on Avalanche Fuji
3. Contract verifies `msg.sender == trustedRelayer`, transfers mUSDC to recipient
4. Records loan in SQLite
5. Returns tx hash and borrowed amount

### Check Balance (anytime)

```bash
cargo run -p zlend-cli -- balance --relayer http://localhost:3000
```

## CLI Reference

| Command | Flags | Description |
|---------|-------|-------------|
| `generate` | `--mainnet` | Generate BIP-39 mnemonic + ZIP-32 keys. Default: testnet |
| `register` | `--relayer <URL>` | Register keys with relayer. Default: `http://localhost:3000` |
| `balance` | `--relayer <URL>` | Check staked ZEC balance and borrow status |
| `scan` | `--relayer <URL> --txid <TXID>` | Verify a Zcash deposit via trial decryption |
| `borrow` | `--relayer <URL> --amount <N> --recipient <0x...>` | Borrow mUSDC against ZEC collateral |

Keys are stored at `~/.zlend/keys.json`.

## API Reference

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Health check. Returns `{"status": "ok"}` |
| POST | `/register` | Register a new position |
| GET | `/balance/{id}` | Get position balance and borrow status |
| POST | `/scan/{id}` | Scan a Zcash tx for deposits |
| POST | `/borrow/{id}` | Borrow against collateral |
| GET | `/positions` | List all registered positions |

### POST /register

```json
// Request
{ "sk": "<hex>", "vk": "<hex>", "ivk": "<hex>", "address": "<hex>" }

// Response (201)
{ "id": "uuid-...", "address": "<hex>" }
```

### GET /balance/{id}

```json
// Response (200)
{
  "id": "uuid-...",
  "address": "<hex>",
  "balance_zat": 150000000,
  "borrowed_zat": 100000000,
  "note_count": 1
}
```

### POST /scan/{id}

```json
// Request
{ "txid": "abc123..." }

// Response (200)
{ "found": true, "value_zat": 150000000, "total_balance_zat": 150000000 }
```

### POST /borrow/{id}

```json
// Request
{ "amount": 100000000, "recipient": "0x1234..." }

// Response (200)
{ "success": true, "tx_hash": "0xabc...", "borrowed": 100000000, "error": null }
```

Error codes: `400` insufficient collateral, `404` position not found, `409` active loan exists.

## Smart Contracts

### ZLendMVP.sol

Minimal lending contract trusting a single relayer address.

- `borrow(borrower, collateralZat, amount, positionId)` — only callable by `trustedRelayer`
- Enforces 150% collateral ratio onchain
- Prevents duplicate active loans per position
- Transfers mUSDC to borrower on success
- `getLoan(positionId)` — query loan state
- `setRelayer(address)` — owner can update relayer

### MockUSDC.sol

ERC-20 token with 6 decimals, minted to the deployer on construction.

### Run Contract Tests

```bash
cd contracts
forge test -vvv
```

### Deploy to Fuji Testnet

```bash
cd contracts
forge script script/Deploy.s.sol --rpc-url $AVALANCHE_FUJI_RPC --broadcast --private-key $DEPLOYER_KEY
```

## Testing

**34 Rust tests + 13 Solidity tests = 47 total**

```bash
# All Rust tests
cargo test

# Only crypto e2e tests (keygen, encryption, trial decryption)
cargo test --test e2e_zcash_flow

# Only API e2e tests (register, scan, borrow lifecycle)
cargo test --test e2e_api_flow

# Only unit tests
cargo test --lib

# Solidity tests
cd contracts && forge test
```

### Test Coverage

| Suite | Tests | Covers |
|-------|-------|--------|
| zlend-core unit | 14 | Key derivation, SK encryption, scan module |
| e2e_zcash_flow | 10 | Real Orchard note encryption + trial decryption, full crypto lifecycle |
| zlend-relayer unit | 4 | SQLite CRUD operations |
| e2e_api_flow | 6 | Full HTTP lifecycle, collateral enforcement, double-borrow prevention |
| ZLendMVP.t.sol | 13 | Borrow, access control, collateral ratio, boundary conditions |

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `RELAYER_SK_PASSPHRASE` | Yes | `zlend-dev-passphrase-change-me` | Passphrase for encrypting spending keys at rest |
| `TATUM_API_KEY` | No | _(empty)_ | Tatum API key for Zcash transaction fetching. Without it, scan uses simulated deposits |
| `RELAYER_EVM_KEY` | No | _(none)_ | Private key for the relayer's EVM signer (Avalanche) |
| `AVALANCHE_FUJI_RPC` | No | _(none)_ | Avalanche Fuji RPC URL |
| `ZLEND_CONTRACT_ADDRESS` | No | _(none)_ | Deployed ZLendMVP contract address |
| `BIND_ADDR` | No | `0.0.0.0:3000` | Relayer listen address |
| `RUST_LOG` | No | `zlend_relayer=info,tower_http=info` | Log level filter |

> The relayer runs without EVM config — borrow will return a simulated tx hash. Without Tatum, scan falls back to a 1.5 ZEC simulated deposit.

## Security Model

### SK Encryption at Rest

The spending key is encrypted before storage using:
- **KDF:** Argon2id (19456 KiB memory, 2 iterations)
- **Cipher:** ChaCha20-Poly1305
- **Format:** `salt(16 bytes) || nonce(12 bytes) || ciphertext`

The passphrase lives only in process memory, never on disk.

### Trust Model

**Fully centralized for MVP.** The relayer is the sole authority on collateral. This is acceptable because:
- Demonstrates the full cross-chain flow (Zcash → Avalanche)
- The contract interface mirrors the production `ZLendContract.borrow()` signature
- Swapping relayer attestation for ZK proof verification is a single-function change

### What This MVP Does NOT Include

- No ZK proofs (Noir/Ultrahonk) — relayer attestation instead
- No FROST threshold signing — encrypted full sk at relayer
- No oracle — hardcoded 1:1 ZAT:token ratio with 150% collateral
- No repay/withdraw flow (borrow only)
- No liquidation
- No full chain scanning (user provides txid)
- No authentication on relayer API

### Upgrade Path

| MVP | v1 |
|-----|----|
| Encrypted SK at relayer | FROST 2-of-3 threshold signing |
| Relayer attestation | Ultrahonk ZK proofs (Noir) |
| Simulated scan | Full chain scanning via lightwalletd |
| No repay | Repay + withdraw flow |
| Hardcoded ratio | Chainlink oracle + liquidation |

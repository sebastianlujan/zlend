# Plan: Remaining MVP Steps (Tier 4–5)

## Context

Tiers 1–3 are complete (240 tests, all green). The cryptographic core is solid:
- ZIP-32 key derivation, SK encryption, trial decryption
- FROST 2-of-3 threshold signing with ceremony sessions, key persistence, nonce WAL
- Distributed Relayer↔Signer ceremony over HTTP
- Vault state machine, authorization tickets, share repair (RTS)
- CLI with 7 commands, SQLite schema with 8 tables

**What's missing**: the integration layer that connects Zcash and Avalanche to close the deposit→borrow→repay→withdraw loop.

### Current State of Stubs

| File | Status |
|------|--------|
| `ogbank-relayer/src/zcash.rs` | 3-line comment stub |
| `ogbank-relayer/src/evm.rs` | 3-line comment stub |
| `ogbank-relayer/src/api.rs` scan_handler | Returns mock `{"found": false}` with TODO at line 132 |
| `ogbank-relayer/src/api.rs` borrow_handler | Does not exist — no route, no handler |
| `contracts/src/` | Empty directory, no foundry.toml |

---

## Tier 4: MVP Integration Layer

### Step 1: Solidity Contracts (Foundry)

Deploy target: Avalanche Fuji testnet. Relayer-attestation model (no ZK proofs in MVP).

**Files to create:**

`MVP-POC/contracts/foundry.toml`
- Solidity 0.8.24, evm_version = "paris", optimizer 200 runs
- remappings for OpenZeppelin

`MVP-POC/contracts/src/OGBankMVP.sol`
- `onlyRelayer` modifier (single trusted address set at deploy)
- `borrow(address borrower, uint256 collateralZat, uint256 amount, bytes32 positionId)` — mints/transfers USDC to borrower, records position
- `repay(bytes32 positionId, uint256 amount)` — borrower repays, emits `RepaymentReceived`
- `FinishPayment(bytes32 positionId, uint256 amountZat, address recipient)` — event for withdrawal listener
- Position struct: `{ borrower, collateralZat, borrowedAmount, repaidAmount, active }`
- `getPosition(bytes32)` view

`MVP-POC/contracts/src/MockUSDC.sol`
- ERC-20 with `mint(address, uint256)` restricted to owner (for testnet)
- 6 decimals

`MVP-POC/contracts/test/OGBankMVP.t.sol`
- Test borrow flow, repay flow, onlyRelayer check, position tracking
- ~8 tests

`MVP-POC/contracts/script/Deploy.s.sol`
- Deploy MockUSDC, deploy OGBankMVP(relayer, mockUSDC), mint initial supply

**Tests:** ~8 Foundry tests

---

### Step 2: Tatum API Client (zcash.rs)

HTTP client for fetching Zcash transaction data via Tatum REST API.

**File:** `MVP-POC/crates/ogbank-relayer/src/zcash.rs`

**Structs:**
- `TatumClient { api_key, base_url, client: reqwest::Client }`
- `TatumTransaction` — deserialized response with `orchard_actions: Vec<OrchardAction>`
- `OrchardAction { cmx, encrypted_note, ephemeral_key, cv, nullifier }`

**Methods:**
- `TatumClient::new(api_key)` — configure with 30s timeout
- `get_transaction(txid) -> Result<TatumTransaction>` — `GET /v3/zcash/transaction/{txid}`
- `extract_orchard_actions(tx) -> Vec<OrchardAction>` — parse compact actions from response

**Env:** `TATUM_API_KEY`

**Tests:** ~4 (client construction, response deserialization from fixture JSON, action extraction, error handling)

**File:** `MVP-POC/crates/ogbank-relayer/src/zcash.rs`

---

### Step 3: Complete scan_handler (api.rs)

Replace the TODO stub at line 132 with real trial decryption.

**Flow:**
1. Fetch tx via `TatumClient::get_transaction(req.txid)`
2. Extract Orchard actions
3. Call `ogbank_core::scan::scan_compact_actions(ivk_bytes, actions)`
4. For each decrypted note: `db::insert_note(conn, position_id, txid, value_zat, nullifier, memo)`
5. Update position balance: `db::update_position_balance(conn, position_id, new_balance)`
6. Return `{ found: true, notes_found: N, total_value_zat: sum }`

**AppState change:** Add `tatum_client: Option<TatumClient>` to `AppState`

**Tests:** ~3 (scan with mock TatumClient, note insertion verified in DB, balance update)

**File:** `MVP-POC/crates/ogbank-relayer/src/api.rs`

---

### Step 4: Alloy EVM Signer (evm.rs)

Interact with OGBankMVP.sol on Avalanche Fuji via alloy.

**File:** `MVP-POC/crates/ogbank-relayer/src/evm.rs`

**Structs:**
- `EvmSigner { provider, wallet, contract_address }`

**Methods:**
- `EvmSigner::new(rpc_url, private_key, contract_address)` — build alloy provider + signer
- `borrow(borrower, collateral_zat, amount, position_id) -> Result<TxHash>` — encode + send `borrow()` call
- `get_position(position_id) -> Result<Position>` — read contract state

**ABI:** Inline sol! macro or ABI JSON for OGBankMVP contract interface

**Env:** `AVAX_RPC_URL`, `RELAYER_PRIVATE_KEY`, `CONTRACT_ADDRESS`

**Tests:** ~3 (signer construction, ABI encoding correctness, error paths)

**File:** `MVP-POC/crates/ogbank-relayer/src/evm.rs`

---

### Step 5: Borrow Handler (api.rs)

Add the missing `POST /borrow/{id}` endpoint.

**Handler:** `borrow_handler`

**Request:** `BorrowRequest { amount: u64, recipient: String }`

**Flow:**
1. Validate position exists and has sufficient collateral (`balance_zat >= amount * LTV_RATIO`)
2. Call `evm_signer.borrow(recipient, collateral_zat, amount, position_id)`
3. Record loan in DB: `db::insert_loan(conn, position_id, amount, recipient, tx_hash)`
4. Update borrowed_zat: `db::update_position_borrowed(conn, position_id, new_borrowed)`
5. Return `{ loan_id, tx_hash, amount, status: "pending" }`

**AppState change:** Add `evm_signer: Option<EvmSigner>` to `AppState`

**Router:** Add `.route("/borrow/:id", post(borrow_handler))`

**Tests:** ~4 (borrow success, insufficient collateral rejected, no position 404, loan recorded in DB)

**File:** `MVP-POC/crates/ogbank-relayer/src/api.rs`

---

### Step 6: Event Listener + ZEC Withdrawal (listener.rs)

Background task that monitors `FinishPayment` events and returns ZEC to users.

**File:** `MVP-POC/crates/ogbank-relayer/src/listener.rs`

**Structs:**
- `EventListener { provider, contract_address, db, zcash_client }`

**Core loop:**
1. Read `event_cursor` from DB (last processed block)
2. `eth_getLogs` for `FinishPayment` events from cursor to latest
3. For each event:
   - Parse `positionId`, `amountZat`, `recipient`
   - Look up position's origin Zcash address
   - Call `z_sendmany` via `ZcashNodeClient` (zcash.rs)
   - Record withdrawal in DB with `zec_status = "submitted"`
   - Update `event_cursor`
4. Sleep 30s, repeat

**Retry:** Exponential backoff (max 5 attempts) for failed `z_sendmany`

**ZcashNodeClient addition to zcash.rs:**
- `ZcashNodeClient::new(rpc_url)` — JSON-RPC client for zcashd
- `z_sendmany(from, to, amount_zat) -> Result<String>` — returns opid

**Integration:** Spawn as `tokio::spawn` in `main.rs` (only if `AVAX_RPC_URL` is set)

**Tests:** ~4 (event parsing, cursor advancement, retry on failure, idempotent processing)

**Files:** `MVP-POC/crates/ogbank-relayer/src/listener.rs`, `MVP-POC/crates/ogbank-relayer/src/zcash.rs`

---

## Tier 5: End-to-End Validation

### Step 7: Integration Tests

Full lifecycle test with mocked external services.

**File:** `MVP-POC/crates/ogbank-relayer/tests/integration.rs`

**Test scenarios:**
1. `test_register_scan_borrow_lifecycle` — register position → mock scan with fixture → borrow against balance
2. `test_vault_create_sign_lifecycle` — create vault → DKG → sign request → verify signature
3. `test_insufficient_collateral` — register → scan small amount → borrow too much → rejected

**Mock strategy:** Use `axum::test` helpers (already used in existing tests). Mock Tatum responses with fixture JSON. Mock EVM calls by making `evm_signer` optional (None = skip EVM call, return mock tx hash).

**Tests:** ~6

---

### Step 8: Config Validation + Startup Checks

**File:** `MVP-POC/crates/ogbank-relayer/src/main.rs`

- Validate required env vars on startup (warn if optional ones missing)
- Log configured mode: `distributed` vs `inline`, EVM enabled/disabled
- Print startup banner with version + bound address

**Scope:** ~50 lines in main.rs

---

## Implementation Order

```
Step 1 (Solidity contracts)
  ↓
Step 2 (Tatum client)     Step 4 (Alloy EVM signer)   ← parallel
  ↓                          ↓
Step 3 (scan_handler)      Step 5 (borrow_handler)     ← parallel
  ↓                          ↓
Step 6 (Event listener — needs both zcash.rs + evm.rs)
  ↓
Step 7 (Integration tests)
  ↓
Step 8 (Config polish)
```

## Test Count Target

| Step | New Tests | Running Total |
|------|-----------|---------------|
| Baseline (Tier 3 done) | — | 240 |
| Step 1 (Foundry) | ~8 | 240 + 8 forge |
| Step 2 (Tatum) | ~4 | 244 |
| Step 3 (scan) | ~3 | 247 |
| Step 4 (EVM) | ~3 | 250 |
| Step 5 (borrow) | ~4 | 254 |
| Step 6 (listener) | ~4 | 258 |
| Step 7 (integration) | ~6 | 264 |
| Step 8 (config) | ~0 | 264 |

## Verification

1. `cargo test --workspace` — all tests pass at each step
2. `cargo clippy --workspace` — no errors
3. `forge test` in contracts/ — Foundry tests pass
4. After Step 3: `POST /scan/{id}` returns real notes from fixture data
5. After Step 5: `POST /borrow/{id}` records loan + calls EVM
6. After Step 6: event listener processes mock FinishPayment events
7. After Step 7: full register→scan→borrow lifecycle test passes

## Key Files

| File | Role |
|------|------|
| `contracts/src/OGBankMVP.sol` | Avalanche lending contract |
| `contracts/src/MockUSDC.sol` | Test ERC-20 token |
| `ogbank-relayer/src/zcash.rs` | Tatum API + zcashd RPC client |
| `ogbank-relayer/src/evm.rs` | Alloy EVM signer |
| `ogbank-relayer/src/listener.rs` | FinishPayment event poller |
| `ogbank-relayer/src/api.rs` | scan_handler + borrow_handler completion |
| `ogbank-relayer/src/main.rs` | Listener spawn + config validation |

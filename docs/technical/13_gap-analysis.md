# ZLend Project: Full Implementation Gap Analysis

```
Document:  13_gap-analysis
Version:   1.0
Date:      2026-03-09
Status:    Approved
Scope:     RFC-OGB-001, Continuation Plan, packages/*, apps/webapp
```

---

## Context

This analysis maps the RFC-OGB-001 specification and the continuation plan against the actual codebase across all three packages (`packages/circuits`, `packages/contracts`, `packages/zcash-integration`) and the webapp (`apps/webapp`). The goal is to identify exactly what percentage of each spec is implemented, what critical bugs exist, and what remains to close the deposit→borrow→repay→withdraw loop end-to-end.

---

## 1. RFC-OGB-001 Coverage: ~68%

### Layer 1: Vault (§4) — 88%

| Section | % | Location | Notes |
|---------|---|----------|-------|
| §4.1 Vault Derivation | 100 | `ogbank-core/src/keys.rs` | ZIP-32 path `m/32'/133'/20295'` |
| §4.2 Key Structure | 100 | `ogbank-core/src/keys.rs` | sk→ask/nk/rivk→fvk→ivk→addr |
| §4.3.1 DKG | 85 | `ogbank-core/src/frost.rs` | 3 rounds implemented, **local-only** (no network transport) |
| §4.3.2 Trusted Dealer | 100 | `ogbank-core/src/frost.rs` | `generate_with_dealer()` |
| §4.3.3 Vault Assembly | 100 | `ogbank-core/src/frost.rs` | vault_id with BLAKE2b domain separator |
| §4.4 Key Distribution | 100 | `ogbank-core/src/frost.rs` | Correct per-participant distribution |
| §4.5 Vault Funding | 50 | `ogbank-relayer/src/api.rs` | Trial decryption works; no real on-chain deposit flow |

### Layer 2: Authorization (§5) — 93%

| Section | % | Location | Notes |
|---------|---|----------|-------|
| §5.1 Auth Tickets | 100 | `ogbank-core/src/auth.rs` | Full ticket lifecycle |
| §5.2 Auth Commitment | 100 | `ogbank-core/src/auth.rs` | BLAKE2b with domain separator |
| §5.3 Auth Merkle Tree | 100 | `ogbank-core/src/auth.rs` | Depth-20, incremental |
| §5.4 Auth Merkle Proof | 100 | `ogbank-core/src/auth.rs` | Verify against root |
| §5.5 Spent Nullifier Set | 40 | `ogbank-core/src/auth.rs` | **HashSet instead of SMT depth-256** |
| §5.6 Ticket Batch | 100 | `ogbank-core/src/protocol.rs` | RelayerTicketBatch + SignerTicketBatch DTOs |

### Layer 3a: Oblivious Sync (§6) — 77%

| Section | % | Location | Notes |
|---------|---|----------|-------|
| §6.2 NullifierWatch | 100 | `ogbank-core/src/sync.rs` | watch/unwatch/scan |
| §6.3 Signer Chain State | 85 | `ogbank-core/src/sync.rs` | VaultSyncState exists, not persistent |
| §6.4 Out-of-Band Notes | 40 | `ogbank-relayer/src/api.rs` | scan works; VaultPaymentRequest not implemented |

### Layer 3b: Relayer Logic (§7) — 61%

| Section | % | Location | Notes |
|---------|---|----------|-------|
| §7.1 Relayer State | 80 | `ogbank-relayer/src/api.rs` | State in API, not monolithic RFC struct |
| §7.2.1 Nonce Pre-Processing | 100 | `ogbank-core/src/nonces.rs` | NonceRegistry + NoncePool + WAL |
| §7.2.2 TX Construction | 20 | — | **No real Orchard Action construction** |
| §7.2.3 Halo2 Proof Gen | 10 | — | No Halo2; uses Noir/UltraHonk instead |
| §7.2.4 FROST Signing | 100 | `ogbank-core/src/ceremony.rs` | Re-randomized FROST complete |
| §7.3 Auth Validation | 100 | `ogbank-core/src/signer.rs` | All 7 checks implemented |
| §7.4 Note Provision | 50 | `ogbank-core/src/protocol.rs` | SpendableNoteSet DTO exists, not full flow |
| §7.5 Batch Spending | 0 | — | Not implemented |

### Layer 4: Cross-Chain (§8) — 35%

| Section | % | Location | Notes |
|---------|---|----------|-------|
| §8.1 Balance PCD Proof | 25 | `packages/circuits/src/main.nr` | Exists but no Merkle membership, no real UTXO verification |
| §8.2 Ethereum Verifier | 70 | `packages/contracts/` | Contract + Verifier work, but **repay nullifier hash mismatch** |
| §8.3 Fallback Attestor | 0 | — | Not implemented |

### Vault Lifecycle (§9) — 100%

| Section | % | Location | Notes |
|---------|---|----------|-------|
| §9.1 State Machine | 100 | `ogbank-core/src/sync.rs` | 6 states, validated transitions |
| §9.2 Revocation | 100 | `ogbank-relayer/src/api.rs` | Passive, active, emergency |
| §9.3 Share Refresh | 100 | `ogbank-core/src/key_refresh.rs` | RTS complete |

### Communication (§10) — 70%

| Section | % | Location | Notes |
|---------|---|----------|-------|
| §10.1 Transport | 25 | `ogbank-relayer/` | HTTP/TLS only. **No Nym, Noise IK, Tor** |
| §10.2 Message Types | 100 | `ogbank-core/src/protocol.rs` | 19 message discriminants |
| §10.3 Serialization | 100 | `ogbank-core/src/protocol.rs` | Wire format + keyed BLAKE2b HMAC |

### Security (§11) — 82%

| Section | % | Location | Notes |
|---------|---|----------|-------|
| §11.1 Nonce Safety | 85 | `ogbank-core/src/nonces.rs` | WAL in-memory only (no disk persistence) |
| §11.2 Signer Availability | 80 | `ogbank-signer/src/main.rs` | Graceful degradation, no replicas |
| §11.4 Front-Running | 100 | `ogbank-core/src/signer.rs` | SIGHASH consistency check |
| §11.5 Privacy Leakage | 65 | — | NullifierWatch ok, oblivious sync not used in practice |

### Test Vectors (§13) — 0%

No companion document exists.

---

## 2. Continuation Plan Coverage: ~72%

| Step | Description | Status | Notes |
|------|-------------|--------|-------|
| Step 1 | Solidity Contracts | **DONE** | `packages/contracts/` — OGBankContract + Verifier + 43 tests |
| Step 2 | Tatum API Client | **DONE** | `ogbank-relayer/src/zcash.rs` — TatumClient complete |
| Step 3 | scan_handler | **DONE** | `ogbank-relayer/src/api.rs` — real trial decryption |
| Step 4 | Alloy EVM Signer | **EMPTY STUB** | `ogbank-relayer/src/evm.rs` — 2 comment lines only |
| Step 5 | Borrow Handler REST | **PARTIAL** | CLI has `borrow` command; no `POST /borrow/{id}` REST endpoint |
| Step 6 | Event Listener | **DONE** | `ogbank-relayer/src/listener.rs` — polling + z_sendmany + retry |
| Step 7 | Integration Tests | **PARTIAL** | Unit tests extensive; no E2E cross-chain lifecycle test |
| Step 8 | Config Validation | **DONE** | `ogbank-relayer/src/main.rs` env var handling |

Test count: **240 Rust tests** (baseline met). Foundry tests: **43 tests**. Target was 264 total Rust tests.

---

## 3. Contracts Analysis (`packages/contracts/`)

### OGBankContract.sol — ~80% functional

**Implemented:**
- `supplyCollateral()` — owner-only, sends to Aave via `supply()`
- `borrow(proof, publicInputs, amount)` — verifies UltraHonk proof, extracts borrow nullifier from `publicInputs[0]`, records it, borrows from Aave, transfers to caller
- `repay(amount, borrowNullifier)` — validates nullifier exists, derives repay nullifier as `keccak256(borrowNullifier, msg.sender)`, repays Aave
- `withdrawProof(proof, publicInputs, amount)` — verifies auth proof, validates full nullifier chain (borrow → repay → not consumed), emits `FinishPayment`
- Full nullifier state machine: `borrowNullifiers`, `repayNullifiers`, `consumedNullifiers`
- Custom errors for all failure cases

**Missing/Broken:**
- **CRITICAL: Repay nullifier hash mismatch** — Contract uses `keccak256(borrowNullifier, msg.sender)` but circuit uses `Poseidon2(user_secret, borrow_nullifier)`. These will **never match** in withdrawProof validation.
- No LTV / collateral ratio enforcement on-chain
- No interest rate tracking or liquidation
- `withdrawProof` emits `FinishPayment` event but does NOT actually withdraw collateral from Aave
- Deploy script: Sepolia addresses are `address(0)` with TODOs; Fuji pool address unverified

### Verifier.sol — 100% (auto-generated)

- Real UltraHonk verifier from `bb write_solidity_verifier`
- VK expects 23 public inputs (UltraHonk backend flattening of 7 circuit public inputs)
- Machine-generated, no manual edits needed

### Test Coverage — Strong

- **Unit tests**: 20 tests covering all functions + error cases
- **MockAavePool tests**: 12 tests (supply/borrow/repay/withdraw)
- **Integration tests**: 11 tests including full lifecycle, multi-user, nullifier replay vectors
- Tests use `vm.mockCall` for verifier (not real proofs)

---

## 4. Circuits Analysis (`packages/circuits/`)

### main.nr — ~40% of RFC §8.1

**Implemented:**
- Dual-mode circuit: mode 0 = Borrow, mode 1 = Auth/Withdraw
- **Mode 0**: `commitment_hash = Poseidon2(secret, value, nonce)`, `nullifier = Poseidon2(secret, nonce)`, constraint `value >= threshold`
- **Mode 1**: `borrow_nullifier = Poseidon2(secret, nonce)`, `repay_nullifier = Poseidon2(secret, borrow_nullifier)`
- 8 passing tests including should_fail cases

**Missing vs RFC:**
- **No Merkle membership proof** — circuit trusts `commitment_hash` as public input, does NOT prove it's in a commitment tree. Anyone can fabricate a commitment.
- **No real UTXO verification** — doesn't verify that ZEC actually exists on Zcash chain
- **No note encryption/decryption verification** — RFC §8.1 specifies proving `pk_d = [ivk] · g_d`
- **No nullifier non-membership proof** — RFC §8.1 requires proving notes are unspent via SMT

### generate-verifier.sh — Complete

Pipeline: `nargo compile` → `bb write_vk` → `bb write_solidity_verifier` → copy to contracts

---

## 5. Webapp Analysis (`apps/webapp/`)

### What's REAL (on-chain):
- **Proof generation** — Actual Noir + Barretenberg UltraHonk proofs (EVM-compatible), runs in browser WASM
- **Borrow tx** — `writeContract({ functionName: "borrow", args: [proof, publicInputs, amount] })`
- **Repay tx** — ERC20 `approve()` then `repay(amount, borrowNullifier)`
- **Withdraw tx** — Generates auth proof (mode 1), calls `withdrawProof(proof, publicInputs, amount)`

### What's SIMULATED:
- **Deposit** — No on-chain tx, 1.5s fake delay, stores in localStorage only
- **Shielded address** — Random string prefixed with "zs1" (not real Zcash address)
- **Interest** — Hardcoded as "~0.00 USDT"
- **Vault storage** — Entirely localStorage (`ogbank-vaults-{address}`), no server/relayer calls
- **Relayer integration** — UI says "relayer will return ZEC" but makes zero API calls to relayer

### Public Input Index Mismatch:
- **Contract** reads `publicInputs[0]` as borrow nullifier (`OGBankContract.sol:118`)
- **Webapp** extracts `publicInputs[2]` as borrow nullifier (`BorrowForm.tsx:45`)
- These may or may not match depending on UltraHonk's flattening order — needs verification

### Data Flow per Step:

```
DEPOSIT (simulated)
  User enters ZEC amount -> localStorage vault created
  vault.userSecret = random 32 bytes
  vault.nonce = random 32 bytes
  vault.shieldedAddress = "zs1" + random (FAKE)
  No Zcash tx, no relayer call

PROOF GENERATION (real, client-side)
  Inputs: vault.userSecret, vault.nonce, zecAmount, wallet address
  commitment_hash = Poseidon2(secret, value, nonce)    [browser WASM]
  nullifier = Poseidon2(secret, nonce)                  [browser WASM]
  UltraHonk proof generated -> stored in vault.proofData

BORROW (real on-chain)
  Calls OGBankContract.borrow(proof, publicInputs, usdtAmount)
  Contract verifies proof, records borrowNullifier
  vault.borrowNullifier = publicInputs[2]   <- INDEX QUESTION
  vault.status = 'borrowed'

REPAY (real on-chain)
  Calls USDT.approve(ogBank, amount)
  Calls OGBankContract.repay(amount, vault.borrowNullifier)
  Contract: repayNullifier = keccak256(borrowNullifier, msg.sender)   <- MISMATCH
  vault.status = 'repaid'

WITHDRAW (real on-chain, BROKEN)
  Generates auth proof mode 1:
    repay_nullifier = Poseidon2(secret, borrowNullifier)   <- DIFFERENT from contract's keccak256
  Calls OGBankContract.withdrawProof(proof, publicInputs, zecAmount)
  Contract checks repayNullifiers[borrowNullifier] exists   <- compares keccak256 result
  Circuit proves Poseidon2 result                           <- NEVER MATCHES
  ** withdrawProof WILL ALWAYS FAIL **
```

---

## 6. Cross-Package Integration Gaps

### Relayer <-> Contracts: ZERO connection
- `evm.rs` is empty (2 comment lines)
- No Alloy provider, no ABI bindings, no contract address config
- No event listener for `FinishPayment` events on Avalanche
- DB schema has `withdrawals` and `event_cursor` tables ready but unused

### Relayer <-> Webapp: ZERO connection
- Webapp stores everything in localStorage
- No API calls to relayer endpoints (`/register`, `/scan`, `/balance`)
- Relayer has 9 REST endpoints that nothing calls

### Circuit <-> Contract: BROKEN
- Repay nullifier derivation: Poseidon2 (circuit) vs keccak256 (contract)
- Public input index: contract reads `[0]`, webapp sends `[2]`
- The `withdrawProof` function will always revert because the nullifier chains diverge at `repay()`

### Zcash <-> Webapp: ZERO connection
- No real shielded addresses
- No actual ZEC deposits
- No Tatum/lightwalletd integration from browser

---

## 7. Critical Bugs (P0)

### Bug 1: Repay Nullifier Hash Mismatch
- **Contract** (`OGBankContract.sol:139`): `keccak256(abi.encodePacked(_borrowNullifier, msg.sender))`
- **Circuit** (`main.nr:44`): `Poseidon2::hash([user_secret, borrow_nullifier])`
- **Impact**: `withdrawProof()` always reverts. The entire withdraw flow is broken.
- **Fix options**:
  - A) Change contract to accept proof-derived repay nullifier instead of computing it
  - B) Change circuit to use keccak256 (expensive in Noir, ~50k constraints)
  - C) Redesign: contract stores circuit-derived repay nullifier during `repay()` call

### Bug 2: Public Input Index Ambiguity
- **Contract** (`OGBankContract.sol:118`): reads `_publicInputs[0]`
- **Webapp** (`BorrowForm.tsx:45`): extracts `publicInputs[2]`
- **Impact**: If indices don't match, borrow nullifier won't be tracked correctly
- **Fix**: Verify UltraHonk's public input flattening order; align contract and webapp

### Bug 3: No Merkle Membership in Circuit
- **Circuit** proves `commitment_hash = Poseidon2(secret, value, nonce)` but doesn't prove this commitment exists in any on-chain tree
- **Impact**: Anyone can fabricate a valid proof for arbitrary amounts without depositing ZEC
- **Fix**: Add Merkle path verification to the circuit (significant work)

---

## 8. What Remains to Close the Loop

### P0 -- Blockers (nothing works E2E without these)

| # | Task | Files | Effort |
|---|------|-------|--------|
| 1 | Fix repay nullifier mismatch | `OGBankContract.sol`, `main.nr`, `WithdrawCard.tsx` | M |
| 2 | Implement `evm.rs` (Alloy signer) | `ogbank-relayer/src/evm.rs` | M |
| 3 | Verify/fix public input index mapping | `OGBankContract.sol`, `BorrowForm.tsx`, `noir.ts` | S |
| 4 | Add `POST /borrow/{id}` REST endpoint | `ogbank-relayer/src/api.rs` | S |

### P1 -- Required for meaningful demo

| # | Task | Files | Effort |
|---|------|-------|--------|
| 5 | Connect webapp to relayer API | `apps/webapp/src/hooks/` | M |
| 6 | Real Zcash shielded address derivation | `apps/webapp/src/hooks/useVaults.ts` | M |
| 7 | Contract event listener (FinishPayment -> DB) | `ogbank-relayer/src/listener.rs` | M |
| 8 | LTV enforcement (on-chain or relayer) | `OGBankContract.sol` or `api.rs` | S |
| 9 | Deploy script: Fuji/Sepolia addresses | `script/Deploy.sol` | S |

### P2 -- Production hardening

| # | Task | Files | Effort |
|---|------|-------|--------|
| 10 | Merkle membership proof in circuit | `main.nr` | L |
| 11 | SpentNullifierSet -> real SMT (depth 256) | `ogbank-core/src/auth.rs` | M |
| 12 | NonceWAL disk persistence | `ogbank-core/src/nonces.rs` | S |
| 13 | Networked DKG (not local-only) | `ogbank-core/src/frost.rs` | M |
| 14 | Real Orchard Action/TX construction | New module | L |
| 15 | E2E integration tests (cross-chain) | `ogbank-relayer/tests/` | M |

### P3 -- Full RFC compliance

| # | Task | Files | Effort |
|---|------|-------|--------|
| 16 | Transport: Noise IK or Tor | New module | L |
| 17 | M-of-N attestor fallback | New contract | L |
| 18 | Test vectors companion document | `docs/` | M |
| 19 | Balance PCD proof (real, per RFC SS 8.1) | `main.nr` rewrite | XL |

---

## 9. Summary Scorecard

```
                          RFC Coverage    Continuation Plan
                          -----------    -----------------
Layer 1 (Vault)           ████████░░ 88%
Layer 2 (Authorization)   █████████░ 93%  <- strongest
Layer 3a (Sync)           ███████░░░ 77%
Layer 3b (Relayer Logic)  ██████░░░░ 61%
Layer 4 (Cross-Chain)     ███░░░░░░░ 35%  <- weakest
Lifecycle                 ██████████ 100%
Communication             ███████░░░ 70%
Security                  ████████░░ 82%
                          -------------
Overall RFC               ██████░░░░ 68%

Steps Done: 1,2,3,6,8     █████░░░░░  5/8
Steps Partial: 5,7         ██░░░░░░░░  2/8
Steps Missing: 4           █░░░░░░░░░  1/8
                          -------------
Overall Plan              ███████░░░ 72%

Contracts                 ████████░░ 80% (functional, 1 critical bug)
Circuits                  ████░░░░░░ 40% (proves preimage, not membership)
Webapp                    ██████░░░░ 60% (proofs real, deposit simulated)
Relayer->Zcash            █████████░ 90% (Tatum + zcashd complete)
Relayer->EVM              ░░░░░░░░░░  0% (evm.rs empty)
Webapp->Relayer           ░░░░░░░░░░  0% (no API calls)
```

**Bottom line**: The cryptographic core (FROST, auth tickets, ceremony, key derivation) is excellent at ~93%. The critical gap is the **integration layer** connecting Zcash <-> EVM end-to-end, with the **repay nullifier hash mismatch** as the most urgent bug -- it makes the withdraw flow impossible.

---

## Verification

To validate this analysis:
1. `cd packages/zcash-integration && cargo test --workspace` -- should pass 240 tests
2. `cd packages/contracts && forge test` -- should pass 43 tests
3. `cd packages/circuits && nargo test` -- should pass 8 tests
4. Grep for `keccak256.*borrowNullifier` in contracts vs `Poseidon2.*borrow_nullifier` in circuit to confirm mismatch
5. Check `evm.rs` is still a stub: `wc -l packages/zcash-integration/crates/ogbank-relayer/src/evm.rs`

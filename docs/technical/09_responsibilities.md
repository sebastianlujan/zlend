# Component Responsibilities

Clear modular boundaries for every component in OGBank. What each owns, what it doesn't, the exact interface between them, and what happens when one fails.

---

## 1. Component Map

```
                         TRUST BOUNDARIES
                         ════════════════

  ┌─────────────────────────────────────────────────────────────────┐
  │                     FULLY TRUSTED (user-controlled)             │
  │                                                                 │
  │  ┌───────────────────────────────────────────────────────────┐  │
  │  │                    BROWSER CLIENT                         │  │
  │  │                                                           │  │
  │  │  sk, ask, nk, rivk, fvk, ivk, ovk, dk, d                │  │
  │  │  FROST shares 1 + 2                                       │  │
  │  │  Noir proof generation (Ultrahonk)                        │  │
  │  │  Trial decryption (WebZjs WASM)                           │  │
  │  │  Avalanche wallet (EVM signing)                           │  │
  │  └───────────┬──────────────────────┬────────────────────────┘  │
  └──────────────┼──────────────────────┼───────────────────────────┘
                 │                      │
    ─ ─ ─ ─ ─ ─ ┼ ─ ─ BOUNDARY 1 ─ ─ ─┼─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─
                 │  (TLS + encrypted    │  (public chain data,
                 │   requests/shares)   │   tx broadcast)
                 ▼                      ▼
  ┌──────────────────────┐   ┌──────────────────────────────────┐
  │   PARTIALLY TRUSTED  │   │          UNTRUSTED               │
  │                      │   │                                  │
  │   OGBANK RELAYER      │   │   EXTERNAL PROVIDERS             │
  │                      │   │                                  │
  │   vk (viewing key)   │   │   Tatum API (JSON-RPC)          │
  │   FROST share 3      │   │   lightwalletd (gRPC:9067)      │
  │   Policy enforcement │   │                                  │
  │   Nonce generation   │   │   Sees: IP, block requests      │
  │   Event monitoring   │   │                                  │
  │   ZEC escrow key     │   │                                  │
  │                      │   │   Cannot: decrypt notes, learn   │
  │                      │   │   which notes belong to user     │
  └──────────┬───────────┘   └──────────────────────────────────┘
             │
    ─ ─ ─ ─ ┼ ─ ─ ─ BOUNDARY 2 ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─
             │  (signed EVM transactions)
             ▼
  ┌──────────────────────────────────────────────────────────────┐
  │                       TRUSTLESS (on-chain)                   │
  │                                                              │
  │  ┌────────────────┐  ┌────────────┐  ┌──────────────────┐  │
  │  │ OGBankContract   │  │ Ultrahonk  │  │ ProtoSocolo      │  │
  │  │                 │──│ Verifier   │  │ (ERC-20)         │  │
  │  │ supply/borrow/  │  │            │  │                  │  │
  │  │ repay/withdraw  │  │ verify()   │  │ ERC20Transfer    │  │
  │  └────────────────┘  └────────────┘  └──────────────────┘  │
  │                                                              │
  │                    AVALANCHE C-CHAIN                         │
  └──────────────────────────────────────────────────────────────┘

  ┌──────────────────────────────────────────────────────────────┐
  │                       TRUSTLESS (on-chain)                   │
  │                                                              │
  │  Commitment tree  ·  Nullifier set  ·  Note encryption      │
  │  Consensus  ·  Memo field transport (512 bytes/note)         │
  │                                                              │
  │                       ZCASH NETWORK                          │
  └──────────────────────────────────────────────────────────────┘
```

---

## 2. Zcash Network

### Owns

| Responsibility | Detail |
|----------------|--------|
| Shielded UTXO storage | Commitment tree (32-level Merkle tree, ~4B notes capacity) |
| Nullifier set | Tracks spent notes, prevents double-spending |
| Note encryption rules | Orchard protocol — notes encrypted with recipient's `ivk` |
| Transaction validity | Consensus — validates proofs, enforces value balance |
| Memo field transport | 512 bytes per note, encrypted with `ivk`, immutable once mined |

### Does NOT Own

| Not Its Job | Who Owns It |
|-------------|-------------|
| Key derivation (`H(X, ZIP32)`) | Browser Client (WebZjs WASM) |
| Trial decryption | Browser Client (`ivk` never leaves browser) |
| FROST share awareness | Nobody — Zcash doesn't know a share is in the memo |
| OGBank protocol logic | Avalanche contracts |
| Collateral valuation | Oracle / off-chain pricing |

---

## 3. OGBank Relayer

### Owns

| Responsibility | Detail |
|----------------|--------|
| FROST share 3 | Holds 1-of-3 — insufficient to sign alone |
| Viewing key (`vk`) | Used to verify collateral exists on Zcash |
| Transaction submission | Signs and submits EVM transactions to Avalanche on user's behalf |
| Policy enforcement | Compliance checks, rate limiting, AML gates — applied before co-signing |
| Nonce generation | Provides fresh nonces for replay protection during `requestUTXOs` |
| Privacy bridge | Breaks the on-chain link between user's Zcash and Avalanche identities |
| Event monitoring | Detects `FinishPayment` events via receipt watching (primary) + `eth_getLogs` polling every 30s (fallback). See [Research §11](06_research.md#11-event-listening--collateral-release-architecture) |
| ZEC collateral release | Sends shielded ZEC back to user's `originAddress` via `z_sendmany` (MVP) or `orchard` crate + lightwalletd (v1) |
| Withdrawal tracking | Maintains `withdrawals` table with `zec_status` (pending → sent → confirmed → failed) and `event_cursor` for last processed block |
| Idempotency | Deduplicates withdrawal processing via `borrow_nullifier` uniqueness + `avax_tx_hash` UNIQUE constraint |
| ZEC retry logic | Retries failed `z_sendmany` calls with exponential backoff (max 5 retries), then marks `requires_manual` |

### Does NOT Own

| Not Its Job | Why |
|-------------|-----|
| Spending key (`sk` or `ask`) | Never sees it — derived and destroyed client-side |
| Shares 1 or 2 | Only holds share 3 |
| ZK proof generation | Proofs are generated entirely in the browser |
| Collateral custody (user's) | User's ZEC stays on Zcash — relayer cannot move it. Relayer only controls the escrow address spending key for collateral release |
| Censorship power | User holds 2-of-3 FROST shares — can always sign alone and self-submit |
| Block scanning | Browser does trial decryption locally |

### Data In / Out

**Receives from Browser:**

| Data | Phase | Purpose |
|------|-------|---------|
| `vk` (viewing key) | Phase 1 | Verify collateral on Zcash |
| FROST share 3 | Phase 0b | Delivered via double-encrypted Zcash memo |
| ZK proof + borrow request | Phase 2 | Forward to `OGBankContract.borrow()` |
| Repay confirmation | Phase 3 | Forward to `OGBankContract.repay()` |
| ZK proof + withdraw request | Phase 4 | Forward to `OGBankContract.withdrawProof()` |
| FROST nonce commitments | Phase 5 | Co-signing round 1 (optional — user can sign alone) |
| FROST partial signature | Phase 5 | Co-signing round 2 (optional) |

**Sends to Browser:**

| Data | Phase | Purpose |
|------|-------|---------|
| UTXO data | Phase 1 | Public chain data for collateral setup |
| Nonce | Phase 1 | Replay protection |
| Nullifier | Phase 1 | Double-collateralization prevention |
| FROST nonce commitment | Phase 5 | Co-signing round 1 (optional) |
| FROST partial signature | Phase 5 | Co-signing round 2 (optional) |

**Sends to Avalanche:**

| Transaction | Phase | Contract Call |
|-------------|-------|--------------|
| Supply collateral | Phase 1 | `OGBankContract.supplyTransfer(amount, utk)` |
| Connect viewing key | Phase 1 | `OGBankContract.connectVk(vk)` |
| Borrow | Phase 2 | `OGBankContract.borrow(proof, amount)` |
| Repay | Phase 3 | `OGBankContract.repay(amount)` |
| Withdraw | Phase 4 | `OGBankContract.withdrawProof(proof, amount)` |

**Receives from Avalanche:**

| Data | Phase | Purpose |
|------|-------|---------|
| `FinishPayment` event (via tx receipt) | Phase 4 | Triggers ZEC collateral release — decoded from `withdraw` tx receipt |
| `FinishPayment` event (via `eth_getLogs`) | Phase 4 | Polling fallback — catches events from direct user submissions or relayer restarts |

**Sends to Zcash:**

| Transaction | Phase | Method |
|-------------|-------|--------|
| Shielded ZEC return (MVP) | Phase 4b | `z_sendmany(escrow → originAddress, amount)` via Zcash node RPC |
| Shielded ZEC return (v1) | Phase 4b | Orchard tx constructed via `orchard` crate + tree state from lightwalletd |

---

## 4. Browser Client

### Owns

| Responsibility | Detail |
|----------------|--------|
| Full key hierarchy | `sk → {ask, nk, rivk} → ak → fvk → {ivk, ovk, dk} → d` |
| FROST shares 1 + 2 | Can sign alone (2-of-3 threshold met) |
| ZK proof generation | Noir circuits compiled to Ultrahonk proofs — entirely client-side |
| Trial decryption | WebZjs WASM scans compact blocks with `ivk` (~5,000 notes/sec) |
| Avalanche wallet | Signs EVM transactions for direct interaction (bypass relayer) |
| FROST DKG ceremony | Generates all 3 shares from `ask`, distributes share 3 to relayer |
| Cold backup | Encrypts share 2 with `age` for offline recovery |

### Does NOT Own

| Not Its Job | Who Owns It |
|-------------|-------------|
| Zcash consensus | Zcash network |
| Avalanche contract state | On-chain (reads only) |
| Relayer's share 3 | Relayer — cannot recover without relayer cooperation |
| Policy enforcement | Relayer's role |
| Block production | Both chains handle this independently |

---

## 5. Interface Contracts

### Browser → Relayer

| Data | Phase | Format | Encryption |
|------|-------|--------|------------|
| `vk` | 1 (setup) | Raw bytes | TLS |
| FROST share 3 | 0b (DKG) | NaCl `crypto_box` inside Zcash memo | Double: NaCl inner + Zcash note outer |
| Borrow request | 2 | `{proof, amount, destination}` | TLS |
| Repay confirmation | 3 | `{amount, txHash}` | TLS |
| Withdraw request | 4 | `{proof, amount}` | TLS |
| FROST signing messages | 5 (optional) | Nonce commitments + partial sigs | TLS |

### Relayer → Avalanche

| Action | Phase | Contract Call |
|--------|-------|--------------|
| Supply collateral | 1 | `supplyTransfer(uint256 amount, bytes utk)` |
| Connect viewing key | 1 | `connectVk(bytes vk)` |
| Borrow | 2 | `borrow(bytes proof, uint256 amount)` |
| Repay | 3 | `repay(uint256 amount)` |
| Withdraw | 4 | `withdrawProof(bytes proof, uint256 amount)` |

### Browser → Zcash

| Action | Phase | Method |
|--------|-------|--------|
| Collateral deposit | 0c | Shielded tx to address `d` |
| Compact block fetch | 1b | lightwalletd gRPC `GetBlockRange` |
| FROST share delivery | 0b | Shielded tx memo to relayer's z-address |
| Spend transaction | 5 | `sendrawtransaction` via Tatum or lightwalletd |

### Avalanche → Browser (Events)

| Event | Phase | Data |
|-------|-------|------|
| `FinishPayment` | 4 | `(ogbank, amount, recipient, originAddress)` |

---

## 6. Failure Modes & Sovereignty

| Failure | Impact | Recovery | User Sovereignty |
|---------|--------|----------|------------------|
| **Relayer offline** | Cannot co-sign, cannot submit to Avalanche | User signs alone (2-of-3) + self-submits to Avalanche | Full control retained |
| **Relayer censors** | Refuses to co-sign or forward transactions | User signs alone + self-submits | Full control retained |
| **Relayer compromised** | Attacker gets `vk` + share 3 | `vk` leak: attacker sees balance (not spend). Share 3: useless alone (1-of-3). Rotate relayer, generate new share 3. | Full control retained |
| **Browser data lost** | Loses `sk`, shares 1+2, `ivk` | Recompute `sk` from identity `X` via `H(X, ZIP32)`. Recover share 2 from `age`-encrypted cold backup. | Recoverable |
| **Cold backup lost** | Share 2 backup unavailable | User still has shares 1+2 in browser. Generate new backup immediately. | No immediate impact |
| **Both browser + backup lost** | Cannot reconstruct 2-of-3 | Share 1 gone + share 2 backup gone = cannot sign. Relayer share 3 alone is useless. **Funds locked on Zcash.** | **Catastrophic — design for prevention** |
| **Tatum/lightwalletd down** | Cannot fetch blocks or broadcast transactions | Switch to alternative provider (multiple exist). No sovereignty loss — only public data. | No impact |
| **Relayer crashes mid-withdraw** | `withdraw` tx sent to Avalanche but `FinishPayment` not processed | Polling fallback detects the event on restart via `event_cursor.last_block`. ZEC sent after recovery. | No fund loss — delayed release |
| **Zcash node unreachable** | `z_sendmany` fails after `FinishPayment` detected | Withdrawal marked `zec_status = 'pending'`, retried with exponential backoff (max 5). | No fund loss — delayed release |
| **Escrow insufficient funds** | `z_sendmany` fails with insufficient balance | Critical alert. Withdrawal marked `failed`. Requires manual ZEC top-up of escrow address. | **Operational — requires intervention** |
| **Zcash network fork** | Commitment tree may diverge temporarily | Wait for resolution, re-scan from fork point | Temporary disruption |
| **Avalanche contract bug** | Funds at risk in OGBankContract | Emergency pause (if implemented). ZEC collateral on Zcash is unaffected. | ZEC safe, ERC-20 position at risk |

---

## 7. Privacy Matrix

What each component can and cannot see:

| Data Item | Browser | Relayer | Tatum / LWD | Avalanche (on-chain) | External Observer |
|-----------|:-------:|:-------:|:-----------:|:-------------------:|:-----------------:|
| `sk` (spending key) | **Yes** | No | No | No | No |
| `ask` (spend auth key) | Destroyed after DKG | No | No | No | No |
| FROST shares 1+2 | **Yes** | No | No | No | No |
| FROST share 3 | No | **Yes** | No | No | No |
| `vk` (viewing key) | **Yes** | **Yes** | No | **Yes**\* | No |
| `ivk` (incoming VK) | **Yes** | No | No | No | No |
| `ovk` (outgoing VK) | **Yes** | No | No | No | No |
| ZEC balance | **Yes** | **Yes** (via `vk`) | No | No | No |
| Zcash address `d` | **Yes** | **Yes** (via `vk`) | No | No | No |
| Which UTXOs are collateral | **Yes** | No | No | No | No |
| Borrow amount | **Yes** | **Yes** | No | **Yes** | **Yes** |
| Repay amount | **Yes** | **Yes** | No | **Yes** | **Yes** |
| Avalanche address | **Yes** | **Yes** | No | **Yes** | **Yes** |
| User's IP address | **Yes** | **Yes** | **Yes** | No | No |
| ZK proof inputs | **Yes** | No | No | No | No |
| ZK proof validity | **Yes** | **Yes** | No | **Yes** | **Yes** |

\* `vk` stored on-chain via `connectVk` — visible to contract but not useful without Zcash chain access to perform trial decryption.

| Withdrawal amount | **Yes** | **Yes** | No | **Yes** | **Yes** |
| ZEC escrow spending key | No | **Yes** (MVP) | No | No | No |
| `originAddress` (ZEC return) | **Yes** | **Yes** | No | **Yes**\*\* | No |

\*\* `originAddress` is emitted in `FinishPayment` — visible on-chain but the shielded ZEC transaction back to this address is not linkable by external observers.

**Key takeaway**: The relayer is the most privileged external component — it sees `vk` (full balance visibility), the user's Avalanche address, and the ZEC escrow spending key (MVP). But it cannot spend user funds (1-of-3 insufficient), cannot forge proofs, and cannot censor (user holds 2-of-3). The escrow key is a custodial risk mitigated in v1 by moving to client-side Orchard tx construction via lightwalletd.

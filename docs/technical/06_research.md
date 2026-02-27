# Open Questions & Research

This document tracks design decisions, research areas, and open questions from the OGBank design phase. Questions are marked as **Resolved**, **Deferred**, or **Open**.

---

## 1. Private ERC-20 Balances

**Question**: How to make ERC-20 balances private on Avalanche?

**Context**: After borrowing from Aave V3, the user's ERC-20 balance is visible on-chain. This partially defeats the privacy goal — an observer can see how much was borrowed even if they can't link it to a ZCash address.

**Research Direction**: **ZAMA** (Fully Homomorphic Encryption)

- [ZAMA](https://www.zama.ai/) provides FHE libraries for encrypted on-chain computation
- `fhEVM` — an EVM extension that supports encrypted state variables
- Potential: ERC-20 token with encrypted balances (`eERC-20`)
- Trade-off: FHE is computationally expensive; may impact gas costs and transaction times

**Findings (Feb 2026)**:

- ZAMA fhEVM launched on **Ethereum mainnet** (Dec 2025). Uses a coprocessor model — no chain modifications needed on the host chain.
- Avalanche C-Chain support expected **H1 2026** (not yet live).
- Gas cost for a single encrypted ERC-20 transfer: ~500K-1M gas (~$0.01-0.05 on Avalanche at 25 nAVAX gas price).
- **Railgun** (zk-SNARK + UTXO model for shielded ERC-20) is the most battle-tested alternative but is **not deployed on Avalanche**. Open-source, so could theoretically be forked.
- **Aztec/Penumbra** are separate chains — not applicable to Avalanche C-Chain.

**Decision**: Use **standard ERC-20 (ProtoSocolo) for MVP**. The core privacy innovation is ZK-verified collateral ownership — private balances are a secondary concern. Plan ZAMA fhEVM integration as a **Phase 2 upgrade** once Avalanche support ships.

**Status**: Resolved — Standard ERC-20 for MVP, ZAMA fhEVM planned for Phase 2.

---

## 2. Replay Attacks on Withdraw

**Question**: Can an attacker replay a withdraw proof after multiple repays?

**Context**: A user who borrows, repays, borrows again, and repays again generates multiple valid repayment proofs. If the withdraw proof verification doesn't properly track which specific borrow-repay cycle is being withdrawn from, an attacker could:

1. Borrow 100 tokens, repay 100 tokens (proof A generated)
2. Borrow 100 tokens again, repay 100 tokens (proof B generated)
3. Submit proof A again to withdraw collateral a second time

**Solution: Nullifier per borrow cycle + state root binding**

Each borrow creates a unique nullifier. Each withdraw must reference and consume a specific nullifier. State root binding provides defense in depth.

This follows proven patterns from Tornado Cash (`mapping(bytes32 => bool) nullifierHashes`), Zcash Orchard (nullifier chaining via `rho`), and Aztec (append-only note hash tree + nullifier tree).

**On Borrow**:
```
borrow_nullifier = H(user_secret, borrow_nonce, state_root)
```
The nullifier is stored in an on-chain set.

**On Withdraw**:
The ZK proof must reference a specific `borrow_nullifier`. The contract checks: nullifier exists AND has not been consumed. Then marks it consumed.

**Noir circuit pseudocode**:
```noir
fn withdraw_proof(
    state_root: pub Field,
    borrow_nullifier: pub Field,
    amount: pub Field,
    user_secret: Field,
    nonce: Field,
    merkle_path: [Field; DEPTH],
) {
    assert(hash(user_secret, nonce, state_root) == borrow_nullifier);
    assert(verify_merkle_path(state_root, merkle_path, ...));
}
```

**Solidity pseudocode**:
```solidity
mapping(bytes32 => bool) public borrowNullifiers;
mapping(bytes32 => bool) public consumedNullifiers;

function borrow(bytes proof, uint256 amount) external {
    bytes32 nullifier = ...; // from proof public inputs
    borrowNullifiers[nullifier] = true;
    // ... Aave interaction
}

function withdrawProof(bytes proof, uint256 amount) external {
    bytes32 nullifier = ...; // from proof public inputs
    require(borrowNullifiers[nullifier], "Unknown cycle");
    require(!consumedNullifiers[nullifier], "Already withdrawn");
    consumedNullifiers[nullifier] = true;
    // ... release collateral
}
```

See also: [Protocol Specification — Nullifier Pattern](02_protocol.md#nullifier-per-borrow-cycle)

**Status**: Resolved — Nullifier-per-borrow-cycle + state root binding.

---

## 3. Threshold Spending Keys (FROST)

**Question**: ~~Can spending keys support homomorphic operations?~~ How should `ask` be split for threshold signing?

**Answer**: Use **FROST** (Flexible Round-Optimized Schnorr Threshold signatures) with the `frost-rerandomized` RedPallas ciphersuite from `ZcashFoundation/frost`.

**Why FROST (not raw Shamir, not BLS, not FHE)**:

| Approach | Reconstructs key? | Zcash-native? | Production library? | Verdict |
|----------|:-:|:-:|:-:|---------|
| Raw Shamir SSS | Yes (vulnerability) | No | Generic | Rejected |
| BLS aggregation | No | No (Orchard uses RedPallas, not BLS) | N/A | Incompatible |
| FHE over curves | No | No | Experimental | Too expensive |
| **FROST (RedPallas)** | **No** | **Yes** | **ZcashFoundation/frost** | **Selected** |

**How it works**:

- FROST operates on `ask` (the spend authorizing key), a Pallas scalar derived from `sk`
- `ask` is split via Distributed Key Generation (DKG) into (2, 3) shares
- Signing requires 2 interactive rounds (commit + sign) — the full `ask` is never reconstructed
- The resulting signature is indistinguishable from a standard Orchard RedPallas spend authorization
- User holds 2 shares (can sign alone), relayer holds 1 share (co-signer), backup holds copy of share 2

**Share distribution**: Relayer's share is delivered via Zcash shielded transaction with the share encoded in the encrypted memo field (see [Protocol — Memo Field Format](02_protocol.md#phase-0b--frost-key-generation--share-distribution)).

**Status**: **Resolved** — FROST selected. See [Protocol](02_protocol.md) and [Architecture](01_architecture.md) for integration details.

---

## 3b. Encryption Schemes for Share Distribution

**Question**: What encryption should be used for distributing FROST shares?

**Answer**: Do NOT use Fernet. Use channel-appropriate encryption:

| Channel | Encryption | Rationale |
|---------|-----------|-----------|
| Zcash memo field | Zcash native note encryption (outer) + NaCl `crypto_box` (inner) | Outer: memo encrypted with recipient's `ivk`. Inner: NaCl provides defense-in-depth against `ovk` compromise. |
| Off-chain transfer | NaCl `crypto_box` (X25519 + XSalsa20-Poly1305) | Asymmetric, authenticated, recipient-specific. Available via `libsodium`/`tweetnacl`. |
| Cold storage backup | `age` encryption | Modern, simple, passphrase-based. |

**Why not Fernet**:

- Fernet is **symmetric** (AES-128-CBC + HMAC-SHA256) — requires a pre-shared secret between both parties
- No asymmetric properties — cannot encrypt "for" a specific recipient using their public key
- AES-128 provides only 128-bit security (256-bit preferred for long-term key material)
- Not a universal standard (implementations exist in Go, Ruby, Rust, Erlang, but it remains niche)
- No forward secrecy

**Memo field defense-in-depth**: Even though Zcash native encryption protects the memo, an inner NaCl `crypto_box` layer is recommended for FROST shares because: (1) anyone with the sender's `ovk` can decrypt the outer layer, (2) no forward secrecy on an immutable ledger, (3) shares are long-lived key material worth double-protecting.

**Status**: **Resolved** — Fernet rejected. NaCl crypto_box for off-chain + inner memo layer, Zcash native encryption for outer memo layer, age for cold storage.

---

## 4. Kohaku Investigation

**Question**: What is Kohaku and how does it apply to OGBank?

**Context**: Referenced in design notes alongside "undercol" (undercollateralization). Appears to be related to:
- Privacy/identity protocols
- Undercollateralization detection mechanisms
- Possibly a protocol or library for privacy-preserving liquidation

**Assigned Research**:
- Franco: Kohaku, undercollateralization patterns
- Seba: Viewing key derivation, `regexp(p, event)` pattern matching

**Findings (Feb 2026)**:

**Kohaku** is the **Ethereum Foundation's official privacy wallet SDK**, unveiled by Vitalik Buterin at Devcon (late 2025). GitHub: [ethereum/kohaku](https://github.com/ethereum/kohaku) — TypeScript (94.1%) + Solidity (5.7%).

Key packages:

| Package | Status | Purpose |
|---------|--------|---------|
| `@kohaku-eth/railgun` | Ready | Railgun privacy protocol integration (shielded transactions) |
| `@kohaku-eth/privacy-pools` | In Development | Privacy Pools protocol (Buterin/Soleimani et al.) |
| `@kohaku-eth/pq-account` | Ready | Post-quantum account abstraction (ERC-4337) |
| `@kohaku-eth/provider` | Ready | Multi-provider abstraction (ethers.js, viem, Colibri) |

Additional features: Helios light client, ORAM private queries, ZK identity proofs (ZK Email), per-dapp account isolation.

**Relevance to OGBank**:

1. **Plugin adapter system** — Kohaku's `Plugin` → `PluginInstance` → `Host` architecture allows new privacy protocols to integrate behind a common interface. OGBank can build an adapter as a new `@kohaku-eth/ogbank` package.
2. **Storage abstraction** — `Host.storage` (key-value) and `Host.keystore` (BIP-32 derivation) provide client-side state persistence for viewing keys, borrow nonces, and nullifiers.
3. **Provider abstraction** — `@kohaku-eth/provider` supports ethers v6, viem, Colibri, and Helios. OGBank uses this for Avalanche C-Chain interaction.
4. **Railgun patterns** — Reference implementation showing note encryption, Merkle tree indexing, and ZK proof generation. OGBank's adapter is simpler (no client-side Merkle tree needed).
5. **Privacy Pools** — `@kohaku-eth/privacy-pools` is still a stub (WIP), but the pattern aligns with OGBank's eventual compliance layer.
6. **NOT a liquidation protocol** — Kohaku operates at the wallet infrastructure layer. Undercollateralization detection is a separate concern.

**Deep-dive findings (Feb 2026)**:

Full codebase analysis conducted — see [research/01_kohaku-codebase.md](../research/01_kohaku-codebase.md) for complete Kohaku architecture documentation.

Adapter feasibility assessed — see [research/03_ogbank-kohaku-adapter.md](../research/03_ogbank-kohaku-adapter.md) for OGBank adapter design, type definitions, and comparison with existing adapters.

**Key conclusions:**
- **Primary motivation: viewing key custody.** The viewing key is the only thing that lets users prove their deposit and claim their ZEC. If the user loses it, they lose their deposit. OGBank cannot hold it — that makes OGBank a single point of total failure (relayer already holds spending key). Kohaku delegates vk custody to the user's wallet, where it's backed up alongside the mnemonic.
- OGBank adapter is ~800 lines of TypeScript (vs Railgun's 46K) — simpler because no client-side Merkle tree or note encryption
- `Host.keystore.deriveAt()` can derive `user_secret` for nullifier computation at path `m/44'/7777'/0'/0'/0`
- ZCash viewing key comes from relayer (not derivable from BIP-32 keystore) — stored in `Host.storage`
- Account recovery path: if vk is lost but wallet mnemonic exists, user can re-request vk from relayer by proving identity via user_secret
- Gap: `Host.storage` is plaintext (mitigated: wallet apps typically encrypt their storage)
- Gap: No cross-chain provider abstraction — ZCash RPC goes through `Host.network.fetch()`
- **Recommendation: Build the adapter.** Viewing key custody is the decisive argument. Ecosystem alignment and low implementation cost are bonuses.

**Undercollateralization detection** (separate concern):
- Use proof-based solvency checks: periodic ZK proofs that `collateral_value >= threshold`
- Timeout-based liquidation flagging if user fails to submit solvency proof within a window
- See [Privacy Model — Liquidation Under Privacy](04_privacy-model.md#liquidation-under-privacy)

**Status**: Resolved — Kohaku is EF's privacy wallet SDK. OGBank adapter feasible and recommended. Full analysis in [research/](../research/).

---

## 5. Viewing Key Generation & Event Matching

**Question**: How exactly are ZCash viewing keys generated, and how are they used to match on-chain events?

**Context**: The protocol needs to:
1. Derive viewing keys deterministically from ZIP-32
2. Use viewing keys to scan Avalanche events and identify which belong to the user
3. Pattern: `regexp(p, event)` — a regex-like pattern match inside a ZK circuit

**Findings**:

### ZIP-32 Key Derivation (Sapling)

```
Master Seed → BLAKE2b-512("ZcashIP32Sapling", S) → I_L (sk), I_R (chain code)
  → PRF^expand(sk, 0x00) → ask (spend authorizing key, scalar)
  → PRF^expand(sk, 0x01) → nsk (nullifier private key, scalar)
  → PRF^expand(sk, 0x02) → ovk (outgoing viewing key, 32 bytes)
  → PRF^expand(sk, 0x10) → dk (diversifier key, 32 bytes)
  → ask * G_spend → ak (spend validating key, Jubjub point)
  → nsk * G_proof → nk (nullifier deriving key, Jubjub point)
  → CRH^ivk(ak, nk) → ivk (incoming viewing key, scalar)
```

**Derivation path**: `m_Sapling / 32' / 133' / 0x4F47'`

The fixed account index `0x4F47'` (ASCII "OG") produces exactly **one key pair (vk, sk) per seed**. The relationship is strictly **1:1** — one user identity = one OGBank Unit. No sub-index derivation exists.

See also: [ZCash Integration — ZIP-32 Derivation Details](05_zcash-integration.md#zip-32-derivation-details)

### noir-zk-regex Status

The [hashcloak/noir-zk-regex](https://github.com/hashcloak/noir-zk-regex) Noir target is **still "coming soon"** per the README (Feb 2026). The Rust compiler backend (`@zk-email/zk-regex-compiler`) exists and was audited by zkSecurity (v2.1.1), but full native Noir circuit generation is in active development.

### Event Matching — MVP vs. Full Privacy

| Approach | Privacy | Complexity | Dependency |
|----------|---------|-----------|------------|
| **Hash-based filtering** (MVP) | Partial — vk holder can match locally | Low | None |
| **ZK regex** (Phase 2) | Full — proves event matches without revealing vk | High | noir-zk-regex Noir support |

**MVP approach**: Events emit `H(vk, event_data)`. The viewing key holder computes the hash locally and matches against on-chain events. No ZK circuit needed for matching — only for collateral proofs.

**Phase 2**: When noir-zk-regex Noir support matures, migrate to circuit-based event matching for full privacy.

**Status**: Resolved — ZIP-32 derivation path documented. Hash-based event filtering for MVP, ZK regex for Phase 2.

---

## 6. Private Balance Approach (eerc20)

**Question**: Should OGBank implement an encrypted ERC-20 (eerc20) for internal accounting?

**Context**: From design notes — "Como hacer privados los saldos? eerc20? cryptografia? → ZAMA"

**Options**:

| Approach | Privacy | Complexity | Gas | Avalanche Ready? |
|----------|---------|-----------|-----|-----------------|
| Standard ERC-20 | None (public balances) | Low | Low | Yes |
| ZAMA fhEVM eerc20 | Full (encrypted balances) | High | High (~500K-1M gas/transfer) | H1 2026 |
| Commitment scheme | Partial (hidden amounts, visible transfers) | Medium | Medium | Yes (custom build) |
| Railgun fork | High (shielded via zk-SNARK UTXO) | Medium-High | Medium | Possible (untested) |

**Decision**: Standard ERC-20 for MVP. The privacy of the collateral source (via ZK proofs) is the core innovation. When ZAMA fhEVM supports Avalanche, re-evaluate for Phase 2.

**Status**: Resolved — Standard ERC-20 for MVP. Re-evaluate when ZAMA fhEVM supports Avalanche.

---

## 7. Viewing Key Disclosure Tradeoff

**Question**: Can we avoid disclosing the full `vk` to the relayer?

**Context**: The current design requires sharing `vk` with the relayer so it can verify collateral. But `vk` gives visibility into ALL incoming transactions for the address — not just the specific collateral deposit. This is more information than strictly needed.

**Research Direction**: A restricted ZK proof that proves "I own UTXOs worth ≥ X at address d" without revealing `vk` itself. The relayer would verify the proof rather than scanning with `vk`.

**Trade-offs**:

- ZK proof approach eliminates relayer's ability to monitor ongoing balance changes (better privacy)
- But removes relayer's ability to independently verify undercollateralization in real-time
- Circuit complexity and proving time need benchmarking

**Status**: Open — needs circuit design and performance analysis.

---

## 8. ZK Proof Architecture — Value Extraction & Circuit Design

**Question**: How does a user obtain the note value (`v`) from encrypted Zcash data, and how can this be proven in a ZK circuit on Avalanche?

**Context**: The protocol needs to: (1) extract `v` from shielded notes using the viewing key, (2) prove `v >= threshold` on Avalanche without revealing `v` or the note, (3) prevent double-collateralization via nullifiers.

### 8a. Value Extraction via Trial Decryption

Each Orchard Action publishes on-chain: `cm` (note commitment), `epk` (ephemeral public key), `C_enc` (encrypted ciphertext, 580 bytes), `cv` (value commitment), `nf` (nullifier), `rk` (randomized verification key).

**Trial decryption** recovers the plaintext `(d, v, rseed, memo)` using `ivk`:

```
Step 1: K_agree = [ivk] * epk                                   // ECDH on Pallas
Step 2: K_sym = BLAKE2b-256("Zcash_OrchardKDF", K_agree || epk) // KDF
Step 3: plaintext = ChaCha20-Poly1305.Decrypt(K_sym, C_enc)     // decrypt
Step 4: if AEAD tag validates → note is ours, parse (d, v, rseed, memo)
        if tag fails → note is NOT ours, skip
```

**Why this works**: The sender encrypted with `[esk] * pk_d`. Since `pk_d = [ivk] * g_d` and `epk = [esk] * g_d`, the ECDH shared secret `[ivk] * epk = [esk] * pk_d` — same result, no secrets revealed.

**Verification**: After decryption, recompute `cm_check = SinsemillaCommit_rcm(g_d, pk_d, v, ρ, ψ)` and verify it matches the on-chain `cm`. This confirms the decrypted value is authentic.

**Performance**: WebZjs (WASM) ~5,000 decryptions/sec in browser. Native Rust (`orchard` crate) significantly faster.

### 8b. ZK Circuit Design — Noir Viability

**The fundamental incompatibility**: Noir (Barretenberg) operates over **BN254**. Orchard operates over **Pallas/Vesta**. These are different prime fields.

| Operation | Orchard native | Noir native | Compatible? |
|-----------|---------------|-------------|:-----------:|
| Sinsemilla hash | Pallas incomplete addition | Not available | **No** |
| Note commitment | SinsemillaCommit on Pallas | Not available | **No** |
| Merkle tree hash | Sinsemilla on Pallas | Poseidon on BN254 | **No** |
| Nullifier PRF | Poseidon on Pallas field | Poseidon on BN254 field | **No** (wrong field) |
| Value comparison | u64 range check | u64 range check | **Yes** |

Verifying native Orchard commitments in Noir would require non-native field arithmetic (~100x constraint overhead). A single Sinsemilla verification could cost 50K-100K+ constraints. Full Orchard note proof: 500K+ constraints, proving time in minutes.

**Three alternatives**:

#### Option A: Simplified Poseidon commitment (MVP — RECOMMENDED)

Don't verify Orchard commitments inside the circuit. Use BN254-native Poseidon:

```noir
fn main(
    // Public inputs
    commitment_hash: pub Field,   // Poseidon(user_secret, value, nonce)
    nullifier: pub Field,         // Poseidon(user_secret, nonce)
    threshold: pub Field,         // minimum collateral in zatoshis

    // Private inputs (witness)
    user_secret: Field,           // derived from sk
    value: Field,                 // from trial decryption
    nonce: Field,                 // borrow cycle nonce
) {
    // Verify commitment
    assert(poseidon::hash([user_secret, value, nonce]) == commitment_hash);
    // Verify nullifier
    assert(poseidon::hash([user_secret, nonce]) == nullifier);
    // Verify collateral sufficiency
    assert(value as u64 >= threshold as u64);
}
```

**~2,000 constraints. Proving: sub-second. On-chain verification: ~200K gas.**

Trade-off: The relayer (who already has `ivk` and performs trial decryption) attests that the `commitment_hash` corresponds to a real Zcash note. This adds no new trust assumption — the relayer is already trusted to scan and verify deposits.

#### Option B: Halo2 (native Pallas/Vesta)

Use Zcash's own proving system. All Orchard gadgets (Sinsemilla, Poseidon, ECC, MerkleCRH) exist natively. But Halo2 proofs are not EVM-verifiable without a SNARK-of-a-SNARK recursive wrapper (what Scroll/zkEVM do). Complexity: very high.

#### Option C: Hybrid recursive (future)

1. Halo2 circuit proves Orchard note ownership (Pallas-native)
2. Noir/Ultrahonk verifies the Halo2 proof recursively (BN254 for EVM)
3. Avalanche contract verifies the final Ultrahonk proof

Architecturally clean but cutting-edge ZK research territory.

**Decision**: **Option A for MVP.** Simplified Poseidon commitment in Noir. The relayer bridges the Orchard↔BN254 gap via attestation. Revisit Option B/C when Halo2↔EVM tooling matures.

### 8c. Oracle Requirements

**The circuit itself needs no oracle.** It is pure computation over its inputs.

**The system needs oracles at two boundaries:**

| Boundary | What's needed | MVP approach | Production approach |
|----------|--------------|-------------|-------------------|
| **Zcash state → Avalanche** | Valid commitment tree root (anchor) | Relayer attestation | Cross-chain bridge or ZK light client |
| **ZEC/USD price** | Current price for collateral ratio | Hardcoded or relayer-provided | Chainlink ZEC/USD (available on Avalanche mainnet) |

```
Trial Decryption         ZK Circuit (Noir)          OGBankContract
────────────────         ─────────────────          ──────────────
ivk + epk + C_enc  →   Proves:                →   Verifies proof
  ↓                      • v >= threshold           + checks anchor (Oracle 1)
  v recovered            • commitment valid          + checks price (Oracle 2)
  (no oracle)            • nullifier correct
                         (no oracle)
```

The ZK proof ensures the user authorized the claim and that `v >= threshold`. The relayer cannot inflate `v` because `user_secret` is private. The nullifier prevents replaying the same collateral.

**Status**: Resolved — Option A (simplified Poseidon circuit in Noir) for MVP. Relayer attestation for anchor. Trial decryption for value extraction.

---

## 9. Borrow-Repay Binding — Replay Attack on Second Loan

**Question**: When a user borrows, repays, and borrows again, how does the contract know which repayment corresponds to which loan? Without this binding, a single repay could be "claimed" against multiple borrow cycles to withdraw collateral multiple times.

**Context**: The existing nullifier-per-borrow-cycle (Section 2) prevents replaying *withdraw* proofs. But Phase 3 (Repay) is currently a plain `Repay(amount)` — an ERC-20 transfer with no cryptographic binding to a specific borrow. This opens a replay attack on the repay side.

### The Attack

```
1. User deposits 10 ZEC as collateral
2. User borrows 100 USDC → borrow_nullifier_1 created on-chain
3. User borrows 100 USDC → borrow_nullifier_2 created on-chain
4. User repays  100 USDC → which loan? Contract doesn't know
5. User withdraws via nullifier_1 → "I repaid loan 1" ✓
6. User withdraws via nullifier_2 → "I repaid loan 2" ✓ ← DOUBLE WITHDRAW, single repay
```

The root cause: no 1:1 binding between a repay transaction and the borrow cycle it settles.

### Solution: Repay Nullifier Chain (MVP — Option A)

Extend the nullifier system so that repay also requires a ZK proof. The repay proof generates a `repay_nullifier` that is cryptographically chained to the specific `borrow_nullifier` it settles.

**Nullifier chain**:

```
borrow_nullifier ──────────────── repay_nullifier ──────────────── withdraw
Poseidon(secret, nonce)     →    Poseidon(secret, borrow_nf)  →   consume both
     │                                  │                              │
     on-chain: created                  on-chain: created              on-chain: consumed
```

Given a `borrow_nullifier`, only one valid `repay_nullifier` exists — and only someone who knows `user_secret` can derive it. This enforces a strict 1:1:1 chain: one borrow → one repay → one withdraw.

**Repay circuit (Noir)**:

```noir
fn repay_proof(
    // Public inputs
    borrow_nullifier: pub Field,    // the borrow being repaid
    repay_nullifier: pub Field,     // new nullifier for this repay

    // Private inputs
    user_secret: Field,             // only the borrower knows this
    borrow_nonce: Field,            // nonce from the original borrow
) {
    // Prove knowledge of the secret behind the borrow
    assert(poseidon::hash([user_secret, borrow_nonce]) == borrow_nullifier);

    // Prove the repay nullifier is deterministically chained
    assert(poseidon::hash([user_secret, borrow_nullifier]) == repay_nullifier);
}
```

**~2,000 constraints** (2 Poseidon hashes). Proving: sub-second. Consistent with the borrow circuit from Section 8b.

**Contract logic (Solidity)**:

```solidity
mapping(bytes32 => bool) public borrowNullifiers;     // existing
mapping(bytes32 => bool) public repayNullifiers;      // NEW
mapping(bytes32 => bool) public consumedNullifiers;   // existing

function repay(bytes calldata proof, uint256 amount) external {
    bytes32 borrowNullifier = ...; // from proof public inputs
    bytes32 repayNullifier = ...;  // from proof public inputs

    require(borrowNullifiers[borrowNullifier], "Unknown borrow");
    require(!repayNullifiers[repayNullifier], "Already repaid");
    require(ultrahonkVerifier.verify(proof), "Invalid proof");

    repayNullifiers[repayNullifier] = true;
    // ... ERC-20 transfer back
}

function withdrawProof(bytes calldata proof, uint256 amount) external {
    bytes32 borrowNullifier = ...;
    bytes32 repayNullifier = ...;

    require(borrowNullifiers[borrowNullifier], "Unknown borrow");
    require(repayNullifiers[repayNullifier], "Not repaid");          // NEW
    require(!consumedNullifiers[borrowNullifier], "Already withdrawn");

    consumedNullifiers[borrowNullifier] = true;
    // ... release collateral
}
```

**Privacy preserved**:

| Data | Visible on-chain? | Why |
|------|:-:|---|
| Repay amount | Yes | Required for ERC-20 transfer |
| Which borrow is being repaid | **No** | The link `borrow_nf → repay_nf` requires `user_secret` to derive |
| User identity | **No** | Relayer submits both transactions |
| user_secret | **No** | Private input in the ZK circuit |

An observer sees a `repay_nullifier` on-chain but cannot determine which `borrow_nullifier` it corresponds to without knowing `user_secret`.

### Upgrade Path: Debt Notes (v1 — Option B)

For v1, replace the flat nullifier chain with a **UTXO-style debt note model**. Each borrow creates a "debt note" committed to an on-chain Merkle tree. Repayment "spends" the debt note by revealing its nullifier via ZK proof.

**Debt note structure**:

```
debt_note = (user_secret, amount_borrowed, borrow_nonce, timestamp)
debt_commitment = Poseidon(user_secret, amount_borrowed, borrow_nonce)
debt_nullifier = Poseidon(user_secret, debt_commitment)
```

**On Borrow**: `debt_commitment` is appended to an on-chain Merkle tree (append-only, depth 16 = 65,536 debt notes max).

**On Repay**: ZK proof proves:
1. I know a `debt_note` whose commitment exists in the tree (Merkle path verification)
2. The `debt_nullifier` is correctly derived from the commitment
3. `repay_amount >= amount_borrowed` (full repayment) OR `change_commitment` is valid (partial repayment)

**On Withdraw**: Same as MVP — proof must show debt nullifier was consumed.

**Third circuit mode required**: Unlike the MVP where repay and withdraw share the same auth proof, Option B requires a **dedicated partial repay circuit (mode 2)** because it introduces constraints that don't exist in modes 0 or 1:

| New constraint (mode 2 only) | Purpose |
|------------------------------|---------|
| Merkle path verification against debt tree root | Prove debt note exists |
| `debt_nullifier = Poseidon(user_secret, debt_commitment)` | Consume old note |
| `change_amount = original_amount - repay_amount` | Arithmetic correctness |
| `change_commitment = Poseidon(user_secret, change_amount, new_nonce)` | Create change note |
| `change_amount >= 0` | Prevent negative debt |

This adds ~8K constraints (Merkle path + 3 Poseidon hashes + range checks). The unified circuit becomes a **3-mode design** in v1: mode 0 (borrow), mode 1 (auth — full repay/withdraw), mode 2 (partial repay with change note).

**Advantages over Option A**:

| Capability | Option A (MVP) | Option B (v1) |
|------------|:-:|:-:|
| Borrow-repay binding | 1:1 chain | 1:1 via debt nullifier |
| Partial repayments | No (full repay only) | Yes (change notes) |
| Repay amount privacy | No (visible ERC-20) | Possible (hidden via value commitment) |
| Multiple borrows per cycle | Yes (separate chains) | Yes (separate notes in tree) |
| Merkle tree required | No | Yes (on-chain, depth 16) |
| Circuit complexity | ~2K constraints | ~10K constraints |
| Anonymity set for repays | Linked to specific borrow (by relayer timing) | All debt notes in tree (larger set) |

**Partial repayment model (Option B only)**:

When a user partially repays, the circuit creates a "change" debt note:

```
Original:  debt_note(amount=100)
Repay 60:  spend debt_note(100) → new debt_note(amount=40)
Repay 40:  spend debt_note(40) → debt fully settled
```

This mirrors Zcash's own UTXO change model — each partial repay spends the old note and creates a new smaller one.

**Implementation cost for Option B**:
- On-chain Merkle tree contract (~200 lines Solidity, proven pattern from Tornado Cash)
- Extended Noir circuit with Poseidon Merkle path verification (~10K constraints, still sub-second proving)
- Debt note management in the relayer's state DB

**Status**: Resolved — **Option A (repay nullifier chain) for MVP implementation.** Option B (debt notes with Merkle tree) planned for **v1 roadmap** to enable partial repayments and enhanced repay privacy.

---

## 10. Unified Circuit Design & Withdraw Analysis

**Question**: Does the withdraw phase need a third ZK circuit (or third mode)? How should the borrow and repay/withdraw circuits be organized — separate circuits or a single unified circuit?

### Withdraw Analysis — NO Third Circuit Needed

The withdraw proof must demonstrate: "I am the owner of this borrow-repay cycle and I authorize the collateral release." Examining the constraints:

| Constraint | Repay proof | Withdraw proof | Identical? |
|-----------|------------|---------------|:----------:|
| `Poseidon(user_secret, borrow_nonce) == borrow_nullifier` | Prove borrow ownership | Prove borrow ownership | **Yes** |
| `Poseidon(user_secret, borrow_nullifier) == repay_nullifier` | Chain repay to borrow | Prove repay was authorized | **Yes** |

The ZK constraints are identical. The difference is entirely in the **contract-side state machine**:

| Check | Repay function | Withdraw function |
|-------|:---:|:---:|
| `borrowNullifiers[borrow_nf] == true` | Required | Required |
| `repayNullifiers[repay_nf] == false` | **Required (creates it)** | — |
| `repayNullifiers[repay_nf] == true` | — | **Required (consumes it)** |
| `consumedNullifiers[borrow_nf] == false` | — | **Required** |
| `consumedNullifiers[borrow_nf] = true` | — | **Sets it** |

The state conditions are mutually exclusive at each step — you MUST repay before you can withdraw, and neither can be replayed. The contract state machine provides complete protection without needing a third proof type.

**Verdict: 2 modes, not 3.** The "auth" proof (mode 1) serves both repay and withdraw. The same proof can be submitted to both functions because the contract enforces the correct ordering through state checks.

### Front-Running Prevention — Recipient Binding

Without a bound recipient, a front-runner could copy a proof from the mempool and submit it to `withdrawProof()` with their own address. Since the ZK proof is valid regardless of who submits it, the collateral would be released to the attacker.

**Solution**: Include `recipient` as a public input in both circuit modes. The contract enforces that tokens (borrow) or collateral (withdraw) go to the address specified in the proof. A front-runner can submit the proof, but the assets still go to the legitimate owner's address.

### Unified Circuit — 2-Mode Architecture

Instead of deploying separate verifier contracts for borrow and repay/withdraw, a single unified circuit uses a `mode` flag with conditional constraints:

```noir
use dep::std::hash::poseidon;

fn main(
    // Public inputs
    mode: pub Field,                  // 0 = borrow, 1 = auth (repay/withdraw)
    commitment_hash: pub Field,       // Poseidon(user_secret, value, nonce) — borrow only
    nullifier: pub Field,             // borrow_nullifier
    threshold: pub Field,             // minimum collateral — borrow only
    repay_nullifier: pub Field,       // Poseidon(user_secret, borrow_nf) — auth only
    recipient: pub Field,             // bound recipient address (anti-front-running)

    // Private inputs (witness)
    user_secret: Field,
    value: Field,                     // note value — borrow only
    nonce: Field,                     // borrow_nonce
) {
    // --- Mode validation ---
    let is_borrow = 1 - mode;        // 1 when mode=0, 0 when mode=1
    let is_auth = mode;              // 0 when mode=0, 1 when mode=1
    assert(mode * (mode - 1) == 0);  // mode must be 0 or 1

    // --- Borrow constraints (active when mode=0, zeroed when mode=1) ---
    let computed_commitment = poseidon::hash([user_secret, value, nonce]);
    assert(is_borrow * (computed_commitment - commitment_hash) == 0);

    let computed_nullifier = poseidon::hash([user_secret, nonce]);
    assert(is_borrow * (computed_nullifier - nullifier) == 0);

    // Threshold check (only meaningful in borrow mode)
    // When mode=1 (auth), is_borrow=0 so this constraint becomes 0 == 0
    assert(is_borrow * (value - threshold) == is_borrow * (value - threshold));
    // Additional: value >= threshold enforced via range check
    if mode == 0 {
        assert(value as u64 >= threshold as u64);
    }

    // --- Auth constraints (active when mode=1, zeroed when mode=0) ---
    let borrow_nf_check = poseidon::hash([user_secret, nonce]);
    assert(is_auth * (borrow_nf_check - nullifier) == 0);

    let repay_nf_check = poseidon::hash([user_secret, nullifier]);
    assert(is_auth * (repay_nf_check - repay_nullifier) == 0);
}
```

**Circuit cost**: ~5 Poseidon hashes + 1 range check + conditional multiplications ≈ **~4,000 constraints**. Proving time: sub-second. On-chain verification: ~250K gas (single Ultrahonk verifier).

### Why Unified > Separate Circuits

| Factor | Separate (2 verifiers) | Unified (1 verifier) |
|--------|:---:|:---:|
| Deployment cost | 2x gas | 1x gas |
| Contract complexity | 2 verifier addresses | 1 verifier address |
| Circuit maintenance | 2 Noir files | 1 Noir file |
| Constraint overhead | ~2K each | ~4K total (slightly more) |
| Proving time | Sub-second each | Sub-second |
| Upgrade coordination | Must upgrade both | Single upgrade |

The unified approach saves deployment gas, simplifies the contract, and reduces the surface area for bugs. The slight constraint overhead (~4K vs ~2K per mode) is negligible for Ultrahonk.

### Complete Contract Flow with Unified Circuit

```solidity
// Single verifier for all operations
IUltrahonkVerifier public immutable verifier;

mapping(bytes32 => bool) public borrowNullifiers;
mapping(bytes32 => bool) public repayNullifiers;
mapping(bytes32 => bool) public consumedNullifiers;

function borrow(bytes calldata proof, uint256 amount) external {
    // Decode public inputs — mode must be 0
    (uint256 mode, , bytes32 borrowNf, , , address recipient) = decodePublicInputs(proof);
    require(mode == 0, "Wrong mode");
    require(verifier.verify(proof), "Invalid proof");
    require(!borrowNullifiers[borrowNf], "Nullifier exists");

    borrowNullifiers[borrowNf] = true;
    // ... Aave supply + borrow → transfer to recipient
}

function repay(bytes calldata proof, uint256 amount) external {
    // Decode public inputs — mode must be 1
    (uint256 mode, , bytes32 borrowNf, , bytes32 repayNf, ) = decodePublicInputs(proof);
    require(mode == 1, "Wrong mode");
    require(verifier.verify(proof), "Invalid proof");
    require(borrowNullifiers[borrowNf], "Unknown borrow");
    require(!repayNullifiers[repayNf], "Already repaid");

    repayNullifiers[repayNf] = true;
    // ... ERC-20 transferFrom(user, contract, amount) → Aave repay
}

function withdraw(bytes calldata proof, uint256 amount) external {
    // Decode public inputs — mode must be 1 (same as repay)
    (uint256 mode, , bytes32 borrowNf, , bytes32 repayNf, address recipient) = decodePublicInputs(proof);
    require(mode == 1, "Wrong mode");
    require(verifier.verify(proof), "Invalid proof");
    require(borrowNullifiers[borrowNf], "Unknown borrow");
    require(repayNullifiers[repayNf], "Not repaid");
    require(!consumedNullifiers[borrowNf], "Already withdrawn");

    consumedNullifiers[borrowNf] = true;
    emit FinishPayment(address(this), amount, recipient, address(0));
    // ... signal collateral release to recipient
}
```

### Security Analysis

| Attack vector | Defense |
|--------------|---------|
| Front-runner copies proof from mempool | `recipient` is a public input — assets go to the address bound in the proof |
| Submit borrow proof to repay/withdraw | `mode` flag mismatch — contract rejects `mode != 1` |
| Submit auth proof to borrow | Contract rejects `mode != 0` |
| Withdraw without repaying | `require(repayNullifiers[repayNf])` fails |
| Double withdraw | `require(!consumedNullifiers[borrowNf])` fails |
| Forge proof for someone else's borrow | Requires `user_secret` (private witness) — only provable via valid ZK proof |

**Status**: Resolved — **Unified 2-mode circuit (borrow + auth).** Withdraw reuses the auth mode. Recipient binding prevents front-running. Single Ultrahonk verifier deployment.

---

## References

### Libraries & Frameworks

| Name | URL | Purpose |
|------|-----|---------|
| ZAMA fhEVM | https://www.zama.ai/ | Fully Homomorphic Encryption for EVM |
| Noir | https://noir-lang.org/ | ZK DSL by Aztec |
| Barretenberg / Ultrahonk | https://github.com/AztecProtocol/barretenberg | ZK proving system |
| zcash-primitives-js | https://github.com/zcash-hackworks/zcash-primitives-js | ZCash crypto primitives in JS |
| noir-zk-regex | https://github.com/hashcloak/noir-zk-regex | ZK regex in Noir (Noir target in development) |
| zk-regex-v2 | https://zk.email/blog/zk-regex-v2 | ZK regex theory |
| SparkLend | https://docs.spark.fi/dev/sparklend/core-contracts/pool#supply | Lending pool reference |
| Kohaku | https://github.com/ethereum/kohaku | EF privacy wallet SDK (Railgun, Privacy Pools, PQ accounts) |
| Railgun | https://www.railgun.org/ | Shielded ERC-20 via zk-SNARKs + UTXO model |
| FROST (ZcashFoundation) | https://github.com/ZcashFoundation/frost | Threshold Schnorr signatures (RedPallas ciphersuite) |
| WebZjs (ChainSafe) | https://github.com/ChainSafe/WebZjs | WASM-compiled Orchard primitives for browser |
| libsodium / tweetnacl | https://github.com/nicknisi/tweetnacl-js | NaCl crypto_box for off-chain share encryption |
| age | https://github.com/FiloSottile/age | Modern file encryption for cold storage backups |

### APIs & RPCs

| Name | URL | Purpose |
|------|-----|---------|
| ZCash RPC | https://zcash-rpc.github.io/ | Official RPC docs |
| Tatum ZCash | https://docs.tatum.io/reference/rpc-zcash-validateaddress | Address validation |
| Tatum UTXOs | https://www.postman.com/blockchainnow/tatum-io/request/kqirish/get-utxos-200 | UTXO fetching |

### Research Papers & Protocols

| Topic | Reference |
|-------|-----------|
| Privacy Pools | Buterin, Soleimani et al. — [SSRN](https://papers.ssrn.com/sol3/papers.cfm?abstract_id=4563364) |
| ZIP-32 | [ZCash Improvement Proposal 32](https://zips.z.cash/zip-0032) — HD key derivation (Sapling) |
| Kohaku | [ethereum/kohaku](https://github.com/ethereum/kohaku) — EF privacy wallet SDK |
| Tornado Cash nullifier pattern | [RareSkills — How Tornado Cash Works](https://rareskills.io/post/how-does-tornado-cash-work) |
| Zcash Orchard nullifiers | [Orchard Design — Nullifiers](https://zcash.github.io/orchard/design/nullifiers.html) |
| Nullifier theory | [Mull Over The Nullifier — HackMD](https://hackmd.io/@liangcc/nullifier) |

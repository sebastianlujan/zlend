# Open Questions & Research

This document tracks unresolved design decisions, active research areas, and open questions from the ZLend design phase.

---

## 1. Private ERC-20 Balances

**Question**: How to make ERC-20 balances private on Avalanche?

**Context**: After borrowing from Aave V3, the user's ERC-20 balance is visible on-chain. This partially defeats the privacy goal — an observer can see how much was borrowed even if they can't link it to a ZCash address.

**Research Direction**: **ZAMA** (Fully Homomorphic Encryption)

- [ZAMA](https://www.zama.ai/) provides FHE libraries for encrypted on-chain computation
- `fhEVM` — an EVM extension that supports encrypted state variables
- Potential: ERC-20 token with encrypted balances (`eERC-20`)
- Trade-off: FHE is computationally expensive; may impact gas costs and transaction times

**Status**: Exploratory — needs feasibility analysis for Avalanche C-Chain compatibility.

---

## 2. Replay Attacks on Withdraw

**Question**: Can an attacker replay a withdraw proof after multiple repays?

**Context**: A user who borrows, repays, borrows again, and repays again generates multiple valid repayment proofs. If the withdraw proof verification doesn't properly track which specific borrow-repay cycle is being withdrawn from, an attacker could:

1. Borrow 100 tokens, repay 100 tokens (proof A generated)
2. Borrow 100 tokens again, repay 100 tokens (proof B generated)
3. Submit proof A again to withdraw collateral a second time

**Mitigation Ideas**:
- **Nullifier per borrow cycle** — Each borrow creates a unique nullifier; each withdraw proof must reference a specific nullifier that gets consumed
- **Sequential nonce** — Each borrow increments a nonce; withdraw proofs must reference the exact nonce
- **State root binding** — Proofs are bound to a specific contract state root, making old proofs invalid after state changes

**Status**: Critical — must be resolved before deployment. Nullifier approach is most likely solution.

---

## 3. Threshold Spending Keys ~~(Homomorphic Spending Keys)~~

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

**Share distribution**: Relayer's share is delivered via Zcash shielded transaction with the share encoded in the encrypted memo field (see [Protocol — Memo Field Format](protocol.md#phase-0b--frost-key-generation--share-distribution)).

**Status**: **Resolved** — FROST selected. See [Protocol](protocol.md) and [Architecture](architecture.md) for integration details.

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

**Question**: What is Kohaku and how does it apply to ZLend?

**Context**: Referenced in design notes alongside "undercol" (undercollateralization). Appears to be related to:
- Privacy/identity protocols
- Undercollateralization detection mechanisms
- Possibly a protocol or library for privacy-preserving liquidation

**Assigned Research**:
- Franco: Kohaku, undercollateralization patterns
- Seba: Viewing key derivation, `regexp(p, event)` pattern matching

**Status**: Investigation needed — no conclusion yet.

---

## 5. Viewing Key Generation & Event Matching

**Question**: How exactly are ZCash viewing keys generated, and how are they used to match on-chain events?

**Context**: The protocol needs to:
1. Derive viewing keys deterministically from ZIP-32
2. Use viewing keys to scan Avalanche events and identify which belong to the user
3. Pattern: `regexp(p, event)` — a regex-like pattern match inside a ZK circuit

**Current Understanding**:
- `vk` is derived from the ZIP-32 key tree (Sapling)
- Event matching uses ZK regex (see [hashcloak/noir-zk-regex](https://github.com/hashcloak/noir-zk-regex))
- The viewing key acts as a "filter" — only events matching the key pattern are visible to the key holder

**Open Sub-Questions**:
- Which specific ZIP-32 derivation path for ZLend addresses?
- How is the regex pattern `p` constructed from the viewing key?
- Performance of noir-zk-regex for on-chain event volumes?

**Status**: Partially resolved — needs implementation prototyping.

---

## 6. Private Balance Approach (eerc20)

**Question**: Should ZLend implement an encrypted ERC-20 (eerc20) for internal accounting?

**Context**: From design notes — "Como hacer privados los saldos? eerc20? cryptografia? → ZAMA"

**Options**:

| Approach | Privacy Level | Complexity | Gas Cost |
|----------|--------------|------------|----------|
| Standard ERC-20 | None (public balances) | Low | Low |
| ZAMA fhEVM eerc20 | Full (encrypted balances) | Very High | Very High |
| Commitment scheme | Partial (hidden amounts, visible transfers) | Medium | Medium |
| Tornado-style mixing | Partial (unlinkable transfers) | Medium | Medium |

**Status**: Needs decision — trade-off between privacy guarantee and implementation complexity.

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

## References

### Libraries & Frameworks

| Name | URL | Purpose |
|------|-----|---------|
| FROST (ZcashFoundation) | https://github.com/ZcashFoundation/frost | Threshold Schnorr signatures (RedPallas ciphersuite) |
| ZAMA fhEVM | https://www.zama.ai/ | Fully Homomorphic Encryption for EVM |
| Noir | https://noir-lang.org/ | ZK DSL by Aztec |
| Barretenberg / Ultrahonk | https://github.com/AztecProtocol/barretenberg | ZK proving system |
| WebZjs (ChainSafe) | https://github.com/ChainSafe/WebZjs | WASM-compiled Orchard primitives for browser (replaces abandoned zcash-primitives-js) |
| libsodium / tweetnacl | https://github.com/nicknisi/tweetnacl-js | NaCl crypto_box for off-chain share encryption |
| age | https://github.com/FiloSottile/age | Modern file encryption for cold storage backups |
| noir-zk-regex | https://github.com/hashcloak/noir-zk-regex | ZK regex in Noir |
| zk-regex-v2 | https://zk.email/blog/zk-regex-v2 | ZK regex theory |
| SparkLend | https://docs.spark.fi/dev/sparklend/core-contracts/pool#supply | Lending pool reference |

### APIs & RPCs

| Name | URL | Purpose |
|------|-----|---------|
| ZCash RPC | https://zcash-rpc.github.io/ | Official RPC docs |
| Tatum ZCash | https://docs.tatum.io/reference/rpc-zcash-validateaddress | Address validation |
| Tatum UTXOs | https://www.postman.com/blockchainnow/tatum-io/request/kqirish/get-utxos-200 | UTXO fetching |

### Research Papers & Protocols

| Topic | Reference |
|-------|-----------|
| Privacy Pools | Buterin, Soleimani et al. — SSR |
| ZIP-32 | ZCash Improvement Proposal 32 — HD key derivation |
| Kohaku | TBD — needs investigation |

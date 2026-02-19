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

## 3. Homomorphic Spending Keys

**Question**: Can spending keys support homomorphic operations?

**Context**: If the spending key could support homomorphic operations, it would enable:
- Threshold signatures without revealing the key
- Multi-party computation for shared collateral positions
- Key rotation without re-collateralization

**Research Direction**: Unknown — marked as `??????` in design notes. This intersects with:
- BLS signatures (aggregatable)
- Shamir's Secret Sharing (threshold schemes)
- FHE over elliptic curve points (computationally expensive)

**Status**: Speculative — needs cryptographic research to determine feasibility.

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

## References

### Libraries & Frameworks

| Name | URL | Purpose |
|------|-----|---------|
| ZAMA fhEVM | https://www.zama.ai/ | Fully Homomorphic Encryption for EVM |
| Noir | https://noir-lang.org/ | ZK DSL by Aztec |
| Barretenberg / Ultrahonk | https://github.com/AztecProtocol/barretenberg | ZK proving system |
| zcash-primitives-js | https://github.com/zcash-hackworks/zcash-primitives-js | ZCash crypto primitives in JS |
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

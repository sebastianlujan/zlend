# Open Questions & Research

This document tracks design decisions, research areas, and open questions from the ZLend design phase. Questions are marked as **Resolved**, **Deferred**, or **Open**.

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

## 3. Homomorphic Spending Keys

**Question**: Can spending keys support homomorphic operations?

**Context**: If the spending key could support homomorphic operations, it would enable:
- Threshold signatures without revealing the key
- Multi-party computation for shared collateral positions
- Key rotation without re-collateralization

**Findings**:

- BLS signatures (on BLS12-381) are aggregatable and support threshold schemes via Shamir's Secret Sharing
- FHE over elliptic curve points is computationally prohibitive today
- Zcash's Sapling spending key (`ask`) is a Jubjub scalar — compatible with Shamir's Secret Sharing for threshold schemes
- Key rotation without re-collateralization would require a "re-key" proof: prove `old_sk` and `new_sk` are authorized for the same collateral

**Decision**: Defer to post-MVP. BLS threshold signatures are the most viable path if multi-sig collateral positions are pursued later. Not needed for core functionality.

**Status**: Deferred — Not needed for MVP. BLS threshold signatures viable for future.

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

**Relevance to ZLend**:

1. **Plugin adapter system** — Kohaku's `Plugin` → `PluginInstance` → `Host` architecture allows new privacy protocols to integrate behind a common interface. ZLend can build an adapter as a new `@kohaku-eth/zlend` package.
2. **Storage abstraction** — `Host.storage` (key-value) and `Host.keystore` (BIP-32 derivation) provide client-side state persistence for viewing keys, borrow nonces, and nullifiers.
3. **Provider abstraction** — `@kohaku-eth/provider` supports ethers v6, viem, Colibri, and Helios. ZLend uses this for Avalanche C-Chain interaction.
4. **Railgun patterns** — Reference implementation showing note encryption, Merkle tree indexing, and ZK proof generation. ZLend's adapter is simpler (no client-side Merkle tree needed).
5. **Privacy Pools** — `@kohaku-eth/privacy-pools` is still a stub (WIP), but the pattern aligns with ZLend's eventual compliance layer.
6. **NOT a liquidation protocol** — Kohaku operates at the wallet infrastructure layer. Undercollateralization detection is a separate concern.

**Deep-dive findings (Feb 2026)**:

Full codebase analysis conducted — see [research/01_kohaku-codebase.md](research/01_kohaku-codebase.md) for complete Kohaku architecture documentation.

Adapter feasibility assessed — see [research/03_zlend-kohaku-adapter.md](research/03_zlend-kohaku-adapter.md) for ZLend adapter design, type definitions, and comparison with existing adapters.

**Key conclusions:**
- **Primary motivation: viewing key custody.** The viewing key is the only thing that lets users prove their deposit and claim their ZEC. If the user loses it, they lose their deposit. ZLend cannot hold it — that makes ZLend a single point of total failure (relayer already holds spending key). Kohaku delegates vk custody to the user's wallet, where it's backed up alongside the mnemonic.
- ZLend adapter is ~800 lines of TypeScript (vs Railgun's 46K) — simpler because no client-side Merkle tree or note encryption
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

**Status**: Resolved — Kohaku is EF's privacy wallet SDK. ZLend adapter feasible and recommended. Full analysis in [research/](research/).

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

**Derivation path**: `m_Sapling / 32' / 133' / account'`
- For ZLend-specific addresses: `m_Sapling / 32' / 133' / account' / zlend_index`

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

**Question**: Should ZLend implement an encrypted ERC-20 (eerc20) for internal accounting?

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

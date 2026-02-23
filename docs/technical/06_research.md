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

**Derivation path**: `m_Sapling / 32' / 133' / account'`
- For OGBank-specific addresses: `m_Sapling / 32' / 133' / account' / ogbank_index`

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

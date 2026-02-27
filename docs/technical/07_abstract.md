# OGBank: Abstract & Architectural Meta-Analysis

## Abstract

**OGBank** is a privacy-preserving cross-chain lending protocol that enables users to collateralize **Zcash shielded UTXOs** and borrow **ERC-20 tokens** on **Avalanche** through **Aave V3** — without revealing the borrower's Zcash address, collateral amount, or identity on-chain.

The protocol introduces **OGBank Units** — deterministic addresses derived from Zcash's ZIP-32 hierarchical key derivation standard via the function `OGBank = H(X, ZIP32) → vk, sk`. A user's identity input `X` is bound to a ZIP-32 path, producing a spending key (`sk`, retained client-side), a viewing key (`vk`, shared with the protocol for verification), and a deterministic shielded address (`d`) for collateral deposits. This derivation is entirely client-side — no on-chain registration, no key escrow.

Collateral ownership is verified via **Ultrahonk zero-knowledge proofs** (Noir circuits compiled through Aztec's Barretenberg proving system). The ZK proof asserts: "I control Zcash notes in the global commitment tree whose total value exceeds the borrow threshold, and I have not double-collateralized them" — without disclosing which notes, their values, or the owner's address. Proofs are generated client-side and verified on-chain by a Solidity verifier contract.

The spend authorizing key (`ask`) — the Pallas scalar that produces RedPallas signatures for Zcash Orchard spends — is split into a **(2, 3) threshold** using **FROST** (Flexible Round-Optimized Schnorr Threshold signatures, RFC 9591) with the `frost-rerandomized` RedPallas ciphersuite from `ZcashFoundation/frost`. The user holds 2 shares (full sovereignty — can sign alone), the relayer holds 1 share (co-signing for policy enforcement, cannot act unilaterally), and a backup copy enables recovery. The full `ask` is never reconstructed after the distributed key generation ceremony. FROST shares are distributed to the relayer via Zcash shielded transaction memos with **double encryption**: an inner NaCl `crypto_box` (X25519 + XSalsa20-Poly1305) layer for defense-in-depth against `ovk` compromise, wrapped by Zcash's native note encryption.

A **privacy-preserving relayer** breaks the on-chain link between the user's Zcash and Avalanche identities by submitting borrow transactions on the user's behalf. The relayer sees the viewing key and request timing but cannot forge proofs, spend collateral, or censor the user (who holds 2-of-3 FROST shares and can always self-submit).

Client-side **trial decryption** via **WebZjs** (ChainSafe's WASM-compiled Rust `orchard` crate, ~5,000 notes/sec) replaces the need for a trusted Zcash full node — the user's incoming viewing key (`ivk`) never leaves the browser. Public chain data (blocks, UTXOs, fee estimates) is served by external providers (Tatum API, Zebra lightwalletd) with no privacy loss.

The protocol operates in five phases: (0) deterministic identity creation and FROST key generation, (1) collateral deposit on Zcash and cross-chain binding to Avalanche, (2) ZK-proven borrow through Aave V3, (3) standard repayment, and (4) ZK-proven withdrawal with collateral release. The invariant `P = Σ(C, W)` ensures protocol solvency — the sum of active claims and processed withdrawals always equals total deposited collateral.

**Current status**: Design-phase specification complete (7 documents, 4 architecture diagrams). Zero implementation. Critical open problems include: the Aave V3 collateral bridging mechanism (how Zcash-chain ZEC becomes Avalanche-chain collateral), Noir circuit design for Orchard-native Sinsemilla/Pallas operations, replay attack prevention on withdraw proofs (nullifier design not finalized), and liquidation mechanism selection under privacy constraints.

**Technology stack**: Zcash Orchard (collateral), Avalanche C-Chain (execution), Aave V3 (lending pool), Noir + Ultrahonk (ZK proofs), FROST/RedPallas (threshold signing), WebZjs (browser primitives), NaCl + age (encryption).

---

## Categorical Meta-Analysis

*For a senior developer stepping into this codebase cold.*

### What This Is

OGBank is a **design-phase, zero-implementation** privacy-preserving lending protocol. The repo contains 7 specification documents, 4 architecture diagrams, and 21 skill modules — but no contracts, no relayer, no client code. Everything below is a categorical breakdown of what exists on paper, what's proven sound, what's hand-waving, and what will actually kill you during implementation.

---

### Category 1: Cryptographic Foundation

**Assessment: Solid — builds on production-grade primitives.**

The protocol stacks four independent cryptographic systems. Each one is battle-tested in isolation; the risk is in how they compose.

| Primitive | Origin | Maturity | OGBank Usage | Risk |
|-----------|--------|----------|-------------|------|
| **ZIP-32 HD derivation** | Zcash spec (2018) | Production | Deterministic address generation (`H(X, ZIP32) → sk, vk, d`) | Low — standard Zcash key derivation |
| **Orchard key tree** | Zcash NU5 (2022) | Production | `sk → {ask, nk, rivk}` parallel derivation, `fvk` composition, `ivk` for scanning | Low — audited, deployed on mainnet |
| **FROST threshold signing** | RFC 9591 + ZIP-312 | Production library, undeployed in Zcash mainnet | 2-of-3 split of `ask` via `frost-rerandomized` RedPallas | **Medium** — library exists (`ZcashFoundation/frost`), but OGBank would be a novel consumer. No prior production deployment of FROST on Orchard `ask` outside the Zcash Foundation's own test vectors. |
| **Ultrahonk (Noir/Barretenberg)** | Aztec Network | Alpha | Client-side proof generation, on-chain verification | **High** — Ultrahonk is evolving rapidly. Breaking changes in Noir/BB versions are common. Circuit design is the hardest unsolved problem here. |

**The critical composition risk**: The ZK circuit (Ultrahonk) must encode Orchard note commitments, Merkle paths, and nullifier checks. This means the circuit needs to implement Sinsemilla hashing (Orchard's commitment scheme) and Pallas curve arithmetic inside Noir. As of today, **no production Noir library provides Sinsemilla or Pallas natively**. This will likely require custom circuit gadgets — the single hardest implementation task in the project.

**What a senior dev needs to know**: The cryptographic *primitives* are fine. The *composition* — Noir circuits over Orchard data structures — is unproven and will be the bottleneck.

---

### Category 2: Trust Architecture

**Assessment: Well-designed with one structural weakness.**

OGBank defines three trust domains with clean boundaries:

```
FULLY TRUSTED          PARTIALLY TRUSTED       TRUSTLESS (ON-CHAIN)
─────────────          ─────────────────       ────────────────────
User Browser           OGBank Relayer           Avalanche Contracts
├ sk (spending key)    ├ vk (viewing key)      ├ OGBankContract
├ ask shares 1+2       ├ ask share 3           ├ Ultrahonk Verifier
├ ivk (scanning)       ├ user's Avax addr      ├ Aave V3 Pool
├ proof generation     ├ request timing        └ ProtoSocolo ERC-20
└ trial decryption     └ policy enforcement
```

**Why the FROST 2-of-3 model works**: The user holds 2 shares and can always act unilaterally. The relayer's single share gives it a co-signing role (useful for compliance gating) but zero unilateral power. If the relayer disappears, the user is unaffected. If the relayer is compromised, the attacker gets 1-of-3 — useless for signing.

**The structural weakness: Viewing key over-disclosure.**

The relayer receives the full `vk`, which means it can scan ALL incoming transactions for the OGBank address — not just the collateral deposit. If a user receives other ZEC payments at the same shielded address, the relayer sees those too. This is documented as an open question ([Research](06_research.md) section 7) but has no mitigation path yet. The proposed fix — a restricted ZK proof of UTXO ownership — would require a second circuit that proves "I own notes worth ≥ X" without revealing `vk`. Doable but adds significant circuit complexity.

**For the senior dev**: The trust model is the strongest part of the design. The FROST share distribution via double-encrypted Zcash memos is elegant — it reuses the very privacy infrastructure that OGBank is building on. The `vk` over-disclosure is a real issue but not a blocker for v1.

---

### Category 3: Data Flow & Protocol Phases

**Assessment: Complete specification, but Phase 1 has a gap.**

The protocol defines 7 phases. Here's the dependency graph with implementation complexity:

```
Phase 0a: Identity Creation          ← WebZjs WASM, client-only, LOW complexity
    │
    ▼
Phase 0b: FROST DKG + Share Dist.   ← FROST lib + Zcash memo, MEDIUM complexity
    │
    ▼
Phase 0c: Collateral Deposit         ← Standard Zcash shielded tx, LOW complexity
    │
    ▼
Phase 1: Collateral Setup            ← Relayer interaction + contract calls, MEDIUM
    │                                    GAP: How does OGBankContract verify
    │                                    that the claimed UTXOs actually exist
    │                                    on Zcash? The spec says "connectVk" but
    │                                    doesn't specify the verification mechanism.
    ▼
Phase 1b: Trial Decryption           ← WebZjs + lightwalletd, MEDIUM complexity
    │
    ▼
Phase 2: Borrow                      ← Noir circuit + on-chain verifier + Aave V3
    │                                    HIGHEST COMPLEXITY: Circuit design
    ▼
Phase 3: Repay                       ← Standard Aave V3 repay, LOW complexity
    │
    ▼
Phase 4: Withdraw                    ← Second Noir circuit + on-chain verify
    │                                    HIGH: Replay attack prevention via
    │                                    nullifiers — design is "most likely"
    │                                    but not finalized
    ▼
Phase 5: FROST Threshold Spend       ← FROST signing (2 rounds), MEDIUM complexity
```

**The Phase 1 gap**: `SupplyTransfer(amount, UTk)` and `connectVk(vk)` are defined as contract calls, but the spec doesn't explain how `OGBankContract` validates that the `UTk` (UTXO token reference) corresponds to a real Zcash UTXO. The contract lives on Avalanche — it can't read the Zcash blockchain. Either:

1. The ZK proof in Phase 2 covers this (the proof asserts UTXO existence), making Phase 1 purely bookkeeping
2. Or there's an implicit oracle / relayer attestation step that isn't specified

This ambiguity will surface immediately during contract implementation.

---

### Category 4: Smart Contract Layer

**Assessment: Interface-only spec. Critical design decisions deferred.**

The [Contracts](03_contracts.md) defines a clean `IOGBankContract` interface:

```solidity
function supplyTransfer(uint256 amount, bytes calldata utk) external;
function connectVk(bytes calldata vk) external;
function borrow(bytes calldata proof, uint256 amount) external;
function repay(uint256 amount) external;
function withdrawProof(bytes calldata proof, uint256 amount) external;
event FinishPayment(address indexed ogbank, uint256 amount, address recipient, address originAddress);
```

**What's missing**:

| Gap | Impact | Difficulty |
|-----|--------|-----------|
| **State management** — How are positions tracked? Mapping of ogbank address to collateral, debt, vk, nullifiers? | Blocks implementation | Medium |
| **Nullifier registry** — How are nullifiers stored and checked for double-collateralization? On-chain mapping? Merkle tree? | Blocks withdraw security | High |
| **Ultrahonk verifier deployment** — The verifier contract is auto-generated from the Noir circuit. Circuit design drives contract design. | Blocks everything | Critical |
| **Aave V3 token routing** — Which ERC-20 does OGBank supply to Aave? OGBank doesn't hold ZEC on Avalanche — so what collateral does it supply? This is the fundamental bridging question. | **Architectural gap** | Critical |
| **Liquidation** — The privacy model describes oracle-based, proof-based, and timeout-based approaches but doesn't commit. No contract function for liquidation. | Blocks economic security | High |

**The Aave V3 bridging question is the elephant in the room.** OGBank's collateral is ZEC on the Zcash chain. Aave V3 on Avalanche expects ERC-20 collateral. The ZK proof proves the user *owns* ZEC — but how does `OGBankContract` supply collateral to Aave if the ZEC isn't on Avalanche? Either:

1. OGBank mints a synthetic "proof-backed" token and supplies that to Aave (requires Aave to list it — unlikely)
2. OGBank maintains its own lending pool (doesn't actually use Aave's pool mechanics, just its interest rate model)
3. The Aave integration is aspirational and the v1 is a standalone pool

This is not addressed in any documentation and is the most significant architectural ambiguity.

---

### Category 5: Privacy Model

**Assessment: Rigorous threat model, but one layer of privacy is fundamentally broken.**

The privacy model correctly identifies what's hidden and what's revealed. The threat model table covers 12 attack vectors with mitigations. The FROST co-signer analysis is thorough.

**The fundamental issue: Borrow amounts are public on Avalanche.**

The spec explicitly states this: "Borrow amount: Visible to Avalanche chain — Required for Aave V3 interaction." This means an observer can see:

- Someone borrowed X tokens at time T
- Someone repaid Y tokens at time T'
- The relayer submitted both transactions

Even though the observer can't link this to a Zcash address, the **timing + amount correlation** across the relayer's transactions is a real deanonymization vector. If a user borrows an unusual amount (e.g., 7,843 USDC), and that same relayer submits one borrow per hour, the anonymity set is essentially 1.

**Privacy Pools partially mitigate this** — by mixing multiple users' transactions, you increase the anonymity set. But the spec's Privacy Pools section is a high-level outline, not a design. It references the Buterin/Soleimani paper but doesn't specify:

- Pool denomination sizes (fixed-size deposits like Tornado, or variable?)
- How membership/exclusion proofs interact with the Ultrahonk circuit
- Whether Privacy Pools are v1 or v2 scope

**Compliance architecture is forward-thinking**: The identity-collateral binding model (ZK attestation attached to the user's OGBank Unit, not to their public address) is well-designed. The strict 1:1 model (one seed = one key pair = one position) simplifies compliance — each user has exactly one attestation to manage.

---

### Category 6: External Dependencies

**Assessment: Well-chosen, one stale reference remaining.**

| Dependency | Type | Status | Risk |
|------------|------|--------|------|
| **WebZjs (ChainSafe)** | WASM library | Active (498 commits) | Low — wraps audited Rust crates |
| **ZcashFoundation/frost** | Rust crate | Active, production | Low — backed by Zcash Foundation |
| **Noir + Barretenberg** | ZK framework | Active, breaking changes | **High** — version lock is critical |
| **Tatum API** | External RPC | Commercial service | Medium — vendor lock-in, rate limits |
| **lightwalletd (Zebra)** | gRPC service | Zcash Foundation maintained | Low |
| **Aave V3 on Avalanche** | Protocol integration | Production, immutable | Low — unless the bridging question invalidates it |
| **libsodium / tweetnacl** | Crypto primitives | Stable | Low |
| **age** | File encryption | Stable | Low |

---

### Category 7: Open Problems (Ranked by Severity)

| # | Problem | Severity | Status | Blocks |
|---|---------|----------|--------|--------|
| 1 | **Aave V3 collateral bridging** — How does ZEC on Zcash become collateral on Avalanche? | **Critical** | Not addressed in any doc | Contract design, Aave integration |
| 2 | **Noir circuit design** — Sinsemilla hashing + Pallas curve inside Noir | **Critical** | Not started | Proof generation, verifier deployment |
| 3 | **Replay attacks on withdraw** | **Critical** | Nullifier approach "most likely" — not finalized | Withdraw security |
| 4 | **Phase 1 verification gap** — How does the contract validate UTXO existence? | **High** | Ambiguous | Contract implementation |
| 5 | **Liquidation mechanism** — Which approach? Oracle, proof, timeout? | **High** | Described but uncommitted | Economic security |
| 6 | **Private ERC-20 balances** — ZAMA FHE feasibility on Avalanche | **Medium** | Exploratory | Full privacy guarantee |
| 7 | **Viewing key over-disclosure** to relayer | **Medium** | Open research question | v2 privacy improvement |
| 8 | **Kohaku investigation** | **Low** | Unresearched | Possibly nothing |
| 9 | **ZK regex performance** at real event volumes | **Low** | Needs benchmarking | Event matching feature |

---

### Category 8: Maturity Assessment

```
PRODUCTION-READY                          NOT STARTED
◄──────────────────────────────────────────────────────────►

Orchard/ZIP-32 ─────┐
FROST library ──────┤
WebZjs WASM ────────┤
Tatum API ──────────┤  ← EXTERNAL DEPS: Production
lightwalletd ───────┤
Aave V3 ────────────┘

                    Spec Complete ──────┐
                    Protocol phases ────┤
                    Trust model ────────┤  ← DESIGN: Complete
                    Privacy model ──────┤
                    Contract interface ─┘

                                        Circuit design ─────┐
                                        OGBankContract ──────┤
                                        ProtoSocolo ────────┤
                                        Relayer service ────┤  ← CODE: Zero
                                        Browser client ─────┤
                                        Privacy Pools ──────┤
                                        Liquidation ────────┘
```

**Bottom line**: This is a well-researched, rigorously specified protocol design. The cryptographic choices are sound. The trust model is elegant. But there are 3 critical unresolved architectural questions (Aave bridging, circuit design, replay prevention) that will determine whether this design is actually implementable. The gap between "specification complete" and "first working prototype" is significant — roughly 70% of the total engineering work remains, and the hardest 70% at that.

---

### Recommended Implementation Order

For a senior dev starting implementation:

1. **Resolve the Aave V3 bridging question first** — This determines whether OGBank is a wrapper around Aave or a standalone pool. Everything else follows from this.
2. **Prototype the Noir circuit** — Build a minimal circuit that proves ownership of a single Orchard note. If Sinsemilla/Pallas aren't feasible in Noir, the entire proof architecture needs rethinking.
3. **Build OGBankContract** — Start with `borrow()` and a hardcoded verifier. Get the Aave interaction working (or the standalone pool).
4. **Build the relayer** — FROST co-signing + transaction submission. The relayer is straightforward once the contracts exist.
5. **Build the browser client** — WebZjs integration, trial decryption, proof generation. This is a shell around the already-solved WebZjs and Noir toolchain.
6. **Add Privacy Pools** — This is a v2 feature. Ship without it.

---

## References

| Specification | URL |
|---------------|-----|
| ZIP-32 (HD Key Derivation) | https://zips.z.cash/zip-0032 |
| ZIP-224 (Orchard Key Components) | https://zips.z.cash/zip-0224 |
| ZIP-302 (Memo Fields) | https://zips.z.cash/zip-0302 |
| ZIP-312 (FROST for Zcash) | https://zips.z.cash/zip-0312 |
| RFC 9591 (FROST Protocol) | https://www.rfc-editor.org/rfc/rfc9591.html |
| Privacy Pools Paper | Buterin, Soleimani et al. (SSR) |

| Library | URL |
|---------|-----|
| ZcashFoundation/frost | https://github.com/ZcashFoundation/frost |
| ChainSafe/WebZjs | https://github.com/ChainSafe/WebZjs |
| Aztec/Barretenberg | https://github.com/AztecProtocol/barretenberg |
| Noir Language | https://noir-lang.org/ |
| hashcloak/noir-zk-regex | https://github.com/hashcloak/noir-zk-regex |

| Service | URL |
|---------|-----|
| Tatum Zcash RPC | https://docs.tatum.io/reference/rpc-zcash |
| Zcash RPC Reference | https://zcash-rpc.github.io/ |
| Zebra lightwalletd | https://zebra.zfnd.org/user/lightwalletd.html |
| SparkLend (Pool Pattern) | https://docs.spark.fi/dev/sparklend/core-contracts/pool#supply |

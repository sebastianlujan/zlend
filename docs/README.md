# ZLend Documentation

**ZLend** is a privacy-preserving lending protocol on **Avalanche** that uses **ZCash** shielded UTXOs as collateral to borrow ERC-20 tokens via **Aave V3**. The protocol introduces **ZLend Units** — deterministic addresses derived from ZCash's ZIP-32 key derivation — and uses **Ultrahonk zero-knowledge proofs** to verify collateral ownership without revealing the user's ZCash identity on-chain.

```
ZLend = H(X, ZIP32) → vk, sk
```

---

## Documentation

| Document | Description |
|----------|-------------|
| [Architecture](architecture.md) | System architecture — Browser, ZCash, Relayer, Avalanche contracts, data flow diagrams |
| [Protocol](protocol.md) | Protocol specification — ZLend Units, key derivation, full user flow (supply → borrow → repay → withdraw) |
| [Smart Contracts](contracts.md) | Contract architecture — ZLendContract, ProtoSocolo (ERC-20), Ultrahonk Verifier, Aave V3 integration |
| [ZCash Integration](zcash-integration.md) | ZCash JSON-RPC interfaces, libraries (WebZjs), ZK tooling (noir-zk-regex), viewing key generation |
| [Privacy Model](privacy-model.md) | Privacy guarantees, relayer model, AML/FT compliance, liquidation under privacy, threat model |
| [Responsibilities](responsibilities.md) | Modular boundaries — what Zcash, Relayer, and Browser own, interface contracts, failure modes, privacy matrix |
| [Research](research.md) | Open questions — ZAMA FHE, replay attacks, homomorphic spending keys, Kohaku, eerc20 |
| [Generate PRP](generate-prp.md) | PRP (Product Requirements Prompt) generation template for feature implementation |

---

## Architecture at a Glance

```
┌──────────┐     ┌──────────┐     ┌────────────────────────────────┐
│  Browser  │────▶│  ZCash   │────▶│      Avalanche C-Chain         │
│           │     │  Node    │     │                                │
│ adapter   │     │ ZIP-32   │     │ ZLendContract ↔ Ultrahonk     │
│ balance   │     │ key      │     │      │                        │
│ relayer   │     │ derivation│    │      ▼                        │
│ privacy   │     │          │     │ Aave V3 (supply/borrow)      │
│ pools     │     │ d, vk, sk│     │      │                        │
│           │     │          │     │      ▼                        │
│           │     │          │     │ ProtoSocolo (ERC-20 transfer) │
└──────────┘     └──────────┘     └────────────────────────────────┘
```

---

## Key Concepts

- **ZLend Unit** — A deterministic address derived from `H(X, ZIP32)` that produces a viewing key (`vk`) and spending key (`sk`). See [Protocol](protocol.md).
- **Ultrahonk Proofs** — ZK proofs generated client-side in Noir, verified on-chain. Prove UTXO ownership without revealing the source. See [Contracts](contracts.md).
- **ZLend Relayer** — Submits transactions to Avalanche on behalf of users, breaking the on-chain link between ZCash and Avalanche identities. See [Privacy Model](privacy-model.md).
- **Nullifiers** — Prevent double-collateralization of the same ZCash UTXOs. See [Research](research.md).

---

## Design Assets

Original architecture diagrams are in [assets/](assets/):

| File | Content |
|------|---------|
| [architecture-overview.png](assets/architecture-overview.png) | Full system architecture diagram |
| [contract-interactions.png](assets/contract-interactions.png) | Smart contract interaction flows |
| [zcash-interfaces-research.png](assets/zcash-interfaces-research.png) | ZCash RPC interfaces and research links |
| [privacy-identity-notes.png](assets/privacy-identity-notes.png) | Privacy, identity, and compliance notes |

---

## Stack

| Layer | Technology |
|-------|-----------|
| Collateral | ZCash (shielded UTXOs, ZIP-32) |
| Execution | Avalanche C-Chain (EVM) |
| Lending Pool | Aave V3 (existing deployment) |
| ZK Proofs | Noir + Ultrahonk (Barretenberg) |
| Token Standard | ERC-20 (ProtoSocolo) |
| Relayer | Custom ZLend Relayer |
| ZK Regex | hashcloak/noir-zk-regex |
| ZCash Primitives | ChainSafe/WebZjs (WASM-compiled Orchard) |

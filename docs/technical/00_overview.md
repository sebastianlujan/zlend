# OGBank Technical Overview

**OGBank** is a privacy-preserving lending protocol on **Avalanche** that uses **ZCash** shielded UTXOs as collateral to borrow ERC-20 tokens via **Aave V3**. The protocol introduces **OGBank Units** — deterministic addresses derived from ZCash's ZIP-32 key derivation — and uses **Ultrahonk zero-knowledge proofs** to verify collateral ownership without revealing the user's ZCash identity on-chain.

```
OGBank = H(X, ZIP32) → vk, sk
```

---

## Documentation

| # | Document | Description |
|---|----------|-------------|
| 01 | [Architecture](01_architecture.md) | System architecture — Browser, ZCash, Relayer, Avalanche contracts, data flow diagrams |
| 02 | [Protocol](02_protocol.md) | Protocol specification — OGBank Units, key derivation, full user flow (supply → borrow → repay → withdraw) |
| 03 | [Smart Contracts](03_contracts.md) | Contract architecture — OGBankContract, ProtoSocolo (ERC-20), Ultrahonk Verifier, Aave V3 integration |
| 04 | [Privacy Model](04_privacy-model.md) | Privacy guarantees, relayer model, AML/FT compliance, liquidation under privacy, threat model |
| 05 | [ZCash Integration](05_zcash-integration.md) | ZCash JSON-RPC interfaces, libraries (zcash-primitives-js), ZK tooling (noir-zk-regex), viewing key generation |
| 06 | [Research](06_research.md) | Resolved research — ZAMA FHE, replay attacks, Kohaku, nullifier patterns, ZIP-32 derivation |
| 07 | [Generate PRP](../generate-prp.md) | PRP (Product Requirements Prompt) generation template for feature implementation |

---

## Architecture at a Glance

```
┌──────────┐     ┌──────────┐     ┌────────────────────────────────┐
│  Browser  │────▶│  ZCash   │────▶│      Avalanche C-Chain         │
│           │     │  Node    │     │                                │
│ adapter   │     │ ZIP-32   │     │ OGBankContract ↔ Ultrahonk     │
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

- **OGBank Unit** — A deterministic address derived from `H(X, ZIP32)` that produces a viewing key (`vk`) and spending key (`sk`). See [Protocol](02_protocol.md).
- **Ultrahonk Proofs** — ZK proofs generated client-side in Noir, verified on-chain. Prove UTXO ownership without revealing the source. See [Contracts](03_contracts.md).
- **OGBank Relayer** — Submits transactions to Avalanche on behalf of users, breaking the on-chain link between ZCash and Avalanche identities. See [Privacy Model](04_privacy-model.md).
- **Nullifiers** — Prevent double-collateralization and replay attacks across borrow cycles. See [Protocol](02_protocol.md#nullifier-per-borrow-cycle).

---

## Design Assets

Original architecture diagrams are in [assets/](../assets/):

| File | Content |
|------|---------|
| [architecture-overview.png](../assets/architecture-overview.png) | Full system architecture diagram |
| [contract-interactions.png](../assets/contract-interactions.png) | Smart contract interaction flows |
| [zcash-interfaces-research.png](../assets/zcash-interfaces-research.png) | ZCash RPC interfaces and research links |
| [privacy-identity-notes.png](../assets/privacy-identity-notes.png) | Privacy, identity, and compliance notes |

---

## Stack

| Layer | Technology |
|-------|-----------|
| Collateral | ZCash (shielded UTXOs, ZIP-32) |
| Execution | Avalanche C-Chain (EVM) |
| Lending Pool | Aave V3 (existing deployment) |
| ZK Proofs | Noir + Ultrahonk (Barretenberg) |
| Token Standard | ERC-20 (ProtoSocolo) |
| Relayer | Custom OGBank Relayer |
| ZK Regex | hashcloak/noir-zk-regex |
| ZCash Primitives | zcash-hackworks/zcash-primitives-js |

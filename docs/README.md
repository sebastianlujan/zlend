# ZLend Documentation

**ZLend** is a privacy-preserving lending protocol on **Avalanche** that uses **ZCash** shielded UTXOs as collateral to borrow ERC-20 tokens via **Aave V3**, verified by **Ultrahonk zero-knowledge proofs**.

---

## Technical

Protocol design, smart contracts, privacy model, and ZCash integration.

| # | Document | Description |
|---|----------|-------------|
| 00 | [Overview](technical/00_overview.md) | Technical overview, architecture diagram, key concepts, stack |
| 01 | [Architecture](technical/01_architecture.md) | System architecture — Browser, ZCash, Relayer, Avalanche contracts, data flow |
| 02 | [Protocol](technical/02_protocol.md) | Protocol spec — ZLend Units, key derivation, full user flow |
| 03 | [Smart Contracts](technical/03_contracts.md) | Contract architecture — ZLendContract, Ultrahonk Verifier, Aave V3 |
| 04 | [Privacy Model](technical/04_privacy-model.md) | Privacy guarantees, relayer model, compliance, liquidation, threat model |
| 05 | [ZCash Integration](technical/05_zcash-integration.md) | ZCash JSON-RPC, ZIP-32, viewing keys, ZK tooling |
| 06 | [Research](technical/06_research.md) | Resolved research — ZAMA FHE, replay attacks, Kohaku, nullifiers |

## Product

Market analysis, user personas, and product strategy.

| # | Document | Description |
|---|----------|-------------|
| 01 | [Overview](product/01_overview.md) | Product overview |
| 02 | [Problem](product/02_problem.md) | Problem statement |
| 03 | [Solution](product/03_solution.md) | Solution design |
| 04 | [Architecture](product/04_architecture.md) | Product architecture |
| 05 | [User Persona](product/05_user-persona.md) | Target users |
| 06 | [User Journey](product/06_user-journey.md) | User flows |
| 07 | [Team](product/07_team.md) | Team structure |
| 08 | [Market Data](product/08_market-data.md) | Market analysis |
| 09 | [Metrics](product/09_metrics.md) | Success metrics |

## Research

Deep-dive investigations into specific technical decisions.

| # | Document | Description |
|---|----------|-------------|
| 01 | [Kohaku Codebase](research/01_kohaku-codebase.md) | Full Kohaku (EF privacy wallet SDK) architecture analysis |
| 02 | [ZLend Data Requirements](research/02_zlend-data-requirements.md) | Data requirements for the ZLend adapter |
| 03 | [ZLend Kohaku Adapter](research/03_zlend-kohaku-adapter.md) | Adapter design, type definitions, feasibility |

## Assets

Architecture diagrams and design notes in [assets/](assets/).

## Templates

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
| [Generate PRP](generate-prp.md) | PRP (Product Requirements Prompt) generation template |

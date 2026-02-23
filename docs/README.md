# OGBank Documentation

**OGBank** is a privacy-preserving lending protocol on **Avalanche** that uses **ZCash** shielded UTXOs as collateral to borrow ERC-20 tokens via **Aave V3**, verified by **Ultrahonk zero-knowledge proofs**.

---

## Technical

Protocol design, smart contracts, privacy model, and ZCash integration.

| # | Document | Description |
|---|----------|-------------|
| 00 | [Overview](technical/00_overview.md) | Technical overview, architecture diagram, key concepts, stack |
| 01 | [Architecture](technical/01_architecture.md) | System architecture — Browser, ZCash, Relayer, Avalanche contracts, data flow |
| 02 | [Protocol](technical/02_protocol.md) | Protocol spec — OGBank Units, key derivation, full user flow |
| 03 | [Smart Contracts](technical/03_contracts.md) | Contract architecture — OGBankContract, Ultrahonk Verifier, Aave V3 |
| 04 | [Privacy Model](technical/04_privacy-model.md) | Privacy guarantees, relayer model, compliance, liquidation, threat model |
| 05 | [ZCash Integration](technical/05_zcash-integration.md) | ZCash JSON-RPC, ZIP-32, viewing keys, ZK tooling |
| 06 | [Research](technical/06_research.md) | Resolved research — ZAMA FHE, replay attacks, FROST, Kohaku, nullifiers |
| 07 | [Abstract](technical/07_abstract.md) | Meta-analysis — cryptography, trust architecture, open problems |
| 08 | [MVP](technical/08_mvp.md) | MVP-POC architecture — Rust crates, sequence diagrams, key derivation |
| 09 | [Responsibilities](technical/09_responsibilities.md) | Modular boundaries, trust domains, failure modes, privacy matrix |

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
| 10 | [Landing](product/10_landing.md) | Landing page content |

## Research

Deep-dive investigations into specific technical decisions.

| # | Document | Description |
|---|----------|-------------|
| 01 | [Kohaku Codebase](research/01_kohaku-codebase.md) | Full Kohaku (EF privacy wallet SDK) architecture analysis |
| 02 | [OGBank Data Requirements](research/02_ogbank-data-requirements.md) | Data requirements for the OGBank adapter |
| 03 | [OGBank Kohaku Adapter](research/03_ogbank-kohaku-adapter.md) | Adapter design, type definitions, feasibility |

## Assets

Architecture diagrams and design notes in [assets/](assets/).

## Analysis

Cross-cutting analysis and design audits.

| # | Document | Description |
|---|----------|-------------|
| 01 | [Skills Analysis](analysis/01_skills-analysis.md) | 14-skill analysis against OGBank design (84K review) |

## Process

Templates and workflows.

| # | Document | Description |
|---|----------|-------------|
| 01 | [Generate PRP](process/01_generate-prp.md) | PRP generation template for feature implementation |

---

## Architecture at a Glance

```
┌──────────┐     ┌──────────┐     ┌────────────────────────────────┐
│  Browser  │────▶│  ZCash   │────▶│      Avalanche C-Chain         │
│           │     │  Node    │     │                                │
│ adapter   │     │ ZIP-32   │     │ OGBankContract ↔ Ultrahonk    │
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

- **OGBank Unit** — A deterministic address derived from `H(X, ZIP32)` that produces a viewing key (`vk`) and spending key (`sk`). See [Protocol](technical/02_protocol.md).
- **Ultrahonk Proofs** — ZK proofs generated client-side in Noir, verified on-chain. Prove UTXO ownership without revealing the source. See [Contracts](technical/03_contracts.md).
- **OGBank Relayer** — Submits transactions to Avalanche on behalf of users, breaking the on-chain link between ZCash and Avalanche identities. See [Privacy Model](technical/04_privacy-model.md).
- **Nullifiers** — Prevent double-collateralization of the same ZCash UTXOs. See [Research](technical/06_research.md).

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
| Relayer | Custom OGBank Relayer |
| ZK Regex | hashcloak/noir-zk-regex |
| ZCash Primitives | ChainSafe/WebZjs (WASM-compiled Orchard) |
| [Generate PRP](process/01_generate-prp.md) | PRP (Product Requirements Prompt) generation template |

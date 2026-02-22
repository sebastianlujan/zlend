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
| [Generate PRP](generate-prp.md) | PRP (Product Requirements Prompt) generation template |

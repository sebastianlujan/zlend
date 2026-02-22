# Team Overview

## The Team

ZLend is built by a two-person team. Both members are fullstack — shared ownership of product, smart contracts, cryptography, frontend, and research.

---

### Franco

| Area | Details |
|------|---------|
| **Role** | Co-founder, fullstack |
| **Focus areas** | Smart contract architecture, ZK circuit design, protocol security |
| **Current work** | System architecture, Noir circuit design, Aave V3 integration planning, relayer escrow design |

### Seba

| Area | Details |
|------|---------|
| **Role** | Co-founder, fullstack |
| **Focus areas** | ZCash integration, viewing key derivation, ZK proof system |
| **Current work** | ZCash RPC integration, key derivation implementation, relayer service design |

---

## Working Model

| Aspect | Approach |
|--------|----------|
| **Structure** | Flat. No hierarchy. Both decide on architecture, product, and priorities together. |
| **Ownership** | Shared. Both can work on any part of the stack — contracts, frontend, research, product. |
| **Communication** | Synchronous. Small enough team that alignment happens naturally. |
| **Decision-making** | Consensus on architecture. Either can ship on implementation details. |

---

## Current Phase

**Design & Architecture** (pre-implementation)

What's been done:
- Complete protocol specification and architecture documentation (7 technical docs)
- Privacy model and threat analysis
- Research questions resolved (replay attacks, ZIP-32 derivation, nullifier patterns)
- Product documentation (overview, problem, solution, personas, journey)
- Escrow-based custody model defined

What's next:
- Smart contract implementation (ZLendContract, Ultrahonk Verifier integration)
- Noir ZK circuit development (deposit proof, repayment proof)
- Relayer/escrow service implementation (ZCash account management, ZEC return flow)
- Frontend/browser client (viewing key management, proof generation)
- Testing and security audit

---

## What the Team Needs to Ship

### Skills We Have
- Smart contract development (Solidity)
- Zero-knowledge cryptography (Noir, Ultrahonk)
- ZCash integration (ZIP-32, viewing keys, shielded transactions)
- Frontend development
- Product design and architecture

### Skills/Resources We Need

| Need | Why | Priority |
|------|-----|----------|
| **Security audit** | ZK circuits and smart contracts handling real funds must be audited before mainnet | P0 — cannot launch without |
| **ZCash ecosystem expertise** | Deep integration with ZCash wallets (Ywallet, Zingo) and community | P1 — important for adoption |
| **Frontend/UX design** | The "normie-friendly" experience requires proper design, not developer UI | P1 — core to our positioning |
| **DevOps / Infrastructure** | Relayer hosting, ZCash node operation, monitoring, uptime guarantees | P1 — relayer is critical infrastructure (holds escrow keys) |
| **Community / Growth** | ZCash community engagement, DeFi community awareness | P2 — after MVP ships |

### Potential Partnerships

| Partner | Value |
|---------|-------|
| **Aave** | Integration validation, co-marketing, potential grant |
| **ZCash Foundation / ECC** | ZCash ecosystem support, community credibility, technical guidance |
| **Aztec / Noir team** | Ultrahonk proving system support, circuit optimization |
| **Avalanche Foundation** | Deployment support, potential grant, ecosystem listing |

---

## Scope: What We Build vs. What We Defer

Per Shape Up's appetite-setting: we define what we are willing to invest before defining what we build. Two people, pre-funding, early stage.

### MVP Scope (Build Now)

| Component | Scope | What It Includes | What It Excludes |
|-----------|-------|-----------------|-----------------|
| **Smart contracts** | Minimal | ZLendContract with deposit/borrow/repay/claim. Single collateral type (ZEC). Single borrow asset (USDC). | Multi-asset, variable rates, governance |
| **ZK circuits** | Minimal | Deposit proof + repayment proof in Noir/Ultrahonk | Solvency proof (use timeout-based liquidation for MVP) |
| **Relayer** | Minimal | Account creation, ZEC escrow, ZEC return | High availability, multi-region, auto-scaling |
| **Frontend** | Functional | Web app with wallet connection, deposit flow, borrow flow, repay/claim | Mobile, onboarding polish, educational content |
| **Liquidation** | Simplified | Timeout-based: if user does not submit solvency proof within window, flag for manual review | Automated liquidation, external liquidators, partial liquidation |
| **Testing** | Testnet only | Avalanche Fuji testnet. ZCash testnet. | Mainnet. Audit (needed before mainnet). |

### Explicitly Deferred (Phase 2+)

- Multi-asset collateral (other privacy coins)
- ZAMA fhEVM encrypted balances
- Automated liquidation with external liquidators
- Privacy Pools compliance layer
- Alex-targeted UX (onboarding polish, educational content, landing page)
- Mobile experience
- Multi-sig escrow keys (threshold signatures)
- Kohaku adapter integration (use direct viewing key management for MVP)

### Why This Scope

Two people cannot ship production-grade ZK circuits, battle-tested smart contracts, a reliable relayer, AND a polished frontend simultaneously. The MVP scope focuses on proving the core mechanism works: deposit ZEC, prove collateral, borrow USDC, repay, get ZEC back. Everything else is optimization.

---

## Timeline Estimate

| Milestone | Target | Dependencies |
|-----------|--------|-------------|
| Smart contract skeleton (ZLendContract + verifier integration) | 4 weeks | Noir circuit interface defined |
| Noir circuits (deposit proof + repayment proof) | 6 weeks (parallel with contracts) | Ultrahonk verifier available |
| Relayer MVP (account creation + ZEC return) | 4 weeks (parallel) | ZCash testnet node running |
| Frontend MVP (wallet connect + deposit + borrow + repay) | 4 weeks (after contracts) | Contract ABI stable |
| Integration testing (Fuji testnet) | 3 weeks | All components deployable |
| **Total to testnet MVP** | **~10-14 weeks** | Assumes 2 people, focused |

**Risk buffer:** ZK circuit development is notoriously unpredictable. Proof generation performance, circuit constraints, and Ultrahonk compatibility issues could add 4-8 weeks.

---

## Funding Strategy

| Path | Status | What It Provides |
|------|--------|-----------------|
| Avalanche Foundation grant | Not applied | Deployment support, ecosystem listing, potential funding |
| ZCash Foundation grant | Not applied | Community credibility, potential funding, technical guidance |
| Hackathon prizes | Active (current phase) | Initial validation, visibility, small funding |
| Self-funded | Current | Unlimited timeline flexibility, zero external pressure |

**What would change with funding:**
- Security audit becomes feasible (currently the single largest blocker to mainnet)
- Part-time -> full-time commitment
- Infrastructure costs covered (ZCash node, relayer hosting, testnet gas)
- Legal review budget (regulatory risk assessment — see [02_problem.md](02_problem.md#regulatory-risk-assessment))

---

## Links

- Full product documentation: [01_overview.md](01_overview.md)
- Metrics and success criteria: [09_metrics.md](09_metrics.md)
- Market data & Avalanche value: [08_market-data.md](08_market-data.md)
- Technical architecture: [../01_architecture.md](../01_architecture.md)
- Research resolutions: [../06_research.md](../06_research.md)

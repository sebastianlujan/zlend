# Team Overview

## The Team

ZLend is built by a two-person team. Both members are fullstack — shared ownership of product, smart contracts, cryptography, frontend, and research.

---

### Franco

| Area | Details |
|------|---------|
| **Role** | Co-founder, fullstack |
| **Focus areas** | Smart contract architecture, ZK circuit design, protocol security |
| **Research assignments** | Kohaku protocol investigation, undercollateralization patterns |
| **Current work** | System architecture, Noir circuit design, Aave V3 integration planning |

### Seba

| Area | Details |
|------|---------|
| **Role** | Co-founder, fullstack |
| **Focus areas** | ZCash integration, viewing key derivation, event matching system |
| **Research assignments** | Viewing key generation (ZIP-32), `regexp(p, event)` pattern matching |
| **Current work** | ZCash RPC integration, key derivation implementation, relayer design |

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
- Research questions resolved (ZAMA fhEVM, replay attacks, Kohaku, ZIP-32 derivation, nullifier patterns)
- Product documentation (overview, problem, solution, personas, journey)
- Phased privacy roadmap defined (MVP → Enhanced Privacy → Advanced Features)

What's next:
- Smart contract implementation (ZLendContract, ProtoSocolo, Ultrahonk Verifier integration)
- Noir ZK circuit development (borrow proof, withdrawal proof)
- Relayer service implementation
- Frontend/browser client
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
| **DevOps / Infrastructure** | Relayer hosting, monitoring, uptime guarantees | P1 — relayer is critical infrastructure |
| **Community / Growth** | ZCash community engagement, DeFi community awareness | P2 — after MVP ships |

### Potential Partnerships

| Partner | Value |
|---------|-------|
| **Aave** | Integration validation, co-marketing, potential grant |
| **ZCash Foundation / ECC** | ZCash ecosystem support, community credibility, technical guidance |
| **Aztec / Noir team** | Ultrahonk proving system support, circuit optimization |
| **Avalanche Foundation** | Deployment support, potential grant, ecosystem listing |
| **Kohaku / EF Privacy team** | Privacy Pools integration, Railgun patterns, compliance framework |

---

## Links

- Full product documentation: [01_overview.md](01_overview.md)
- Technical architecture: [../01_architecture.md](../01_architecture.md)
- Research resolutions: [../06_research.md](../06_research.md)

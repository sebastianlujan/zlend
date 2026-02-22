# ZLend — Product Overview

## One-Liner

**ZLend lets you borrow USDC using your ZCash as collateral — without anyone seeing what you own.**

Your collateral stays private. Your loan is real. The math checks out, but your identity stays yours.

---

## Diagnosis

Two realities collide in crypto today:

1. **DeFi lending is a glass house.** If you borrow on Aave, Compound, or any major lending protocol, your entire financial position is public. How much collateral you have, what your liquidation price is, when you're underwater — all visible to anyone with a block explorer. This isn't a bug. It's how these protocols work. But it means sophisticated actors (MEV bots, liquidation hunters, on-chain analysts) can front-run your positions, target your liquidations, and profile your net worth.

2. **Privacy coins are locked out of DeFi.** ZCash has the strongest privacy technology in crypto — shielded transactions that completely hide sender, receiver, and amount. But ZEC holders can't use their assets productively. There's 5.1M ZEC (~$255M) sitting in shielded pools — growing 5x in under two years — with zero DeFi access. 290K ZEC is already wrapped on other chains (Solana, BSC, Near), proving cross-chain demand, but Avalanche has captured none of it.

The result: **people who care most about financial privacy are excluded from the most useful thing in crypto (lending), and people who use lending have zero privacy.**

---

## Guiding Policy

**Build the first lending protocol where privacy is the default, not an add-on.**

We bridge ZCash's proven privacy technology with Avalanche's DeFi infrastructure (Aave V3). Zero-knowledge proofs verify that you have sufficient collateral without revealing what you own or who you are. The user experience should feel like a simple lending app — the cryptography is invisible.

**What this means in practice:**
- Privacy is not optional — it's how the protocol works
- The complexity stays behind the scenes — users see "Deposit, Prove, Borrow"
- We build on existing battle-tested infrastructure (Aave V3, ZCash Sapling) rather than reinventing lending or privacy
- We target crypto-curious users who care about privacy but don't want to learn cryptography
- ZLend operates as a **trusted escrow** — the protocol holds your ZCash during the loan and returns it when you repay

**What we're NOT doing:**
- Not building a privacy mixer (we're a lending protocol)
- Not building a DEX or trading platform
- Not building a new L1/L2 chain
- Not requiring users to understand ZK proofs, nullifiers, or key derivation
- Not competing with Aave — we build on top of it
- Not ignoring regulatory risk — we've assessed the Tornado Cash precedent, OFAC sanctions risk, and FinCEN privacy coin scrutiny. See [Regulatory Risk Assessment](02_problem.md#regulatory-risk-assessment)

---

## Coherent Actions (Sequenced)

The actions are ordered by dependency. Each step enables the next.

1. **ZCash escrow service (Foundation)** — The ZLend relayer manages ZCash accounts with spending keys. Users receive a viewing key to verify their deposit. This is the first thing to build because nothing else works without it. _Validates: Can we reliably create escrow addresses and return ZEC?_
2. **ZK-verified collateral (Core Innovation)** — Users generate zero-knowledge proofs (using their viewing key) that demonstrate sufficient collateral without revealing amounts or addresses. Depends on (1): proofs reference the escrow deposit. _Validates: Can we generate and verify proofs in acceptable time?_
3. **Smart contracts on Avalanche (Integration)** — ZLendContract verifies proofs and orchestrates borrows via Aave V3's lending pool (USDC). Depends on (2): contract needs a working proof format to verify. _Validates: Does the end-to-end flow work on-chain?_
4. **On-chain repayment verification (Completion)** — When a user repays their USDC loan and proves repayment on Avalanche, the protocol automatically releases their ZCash back. Depends on (1-3): requires the full loop to be operational. _Validates: Can users complete the full cycle and get their ZEC back?_

---

## Vision

**Phase 1 (MVP):** Borrow USDC privately using ZCash collateral on Avalanche. User deposits ZEC into a ZLend escrow address, generates a ZK proof of deposit, borrows from Aave V3, repays, and claims their ZEC back. See [scope](07_team.md#scope-what-we-build-vs-what-we-defer) for what's included and excluded.

**Success gate for Phase 2:** 100 deposits from ZCash-native users, zero collateral loss events, completed security audit, positive ZCash community sentiment. See [Metrics Framework](09_metrics.md#phase-1-success-criteria) for full criteria.

**Phase 2 direction (deferred until Phase 1 gate is met):** Multi-asset private collateral (not just ZCash). Multi-sig or threshold control of escrow keys for reduced trust assumptions. Support for additional stablecoins and lending pools beyond Aave V3. These are scoped only after Phase 1 validation.

---

## Links

- Technical architecture: [../01_architecture.md](../01_architecture.md)
- Protocol specification: [../02_protocol.md](../02_protocol.md)
- Problem deep-dive: [02_problem.md](02_problem.md)
- Proposed solution: [03_solution.md](03_solution.md)
- Market data & Avalanche value: [08_market-data.md](08_market-data.md)
- Metrics framework: [09_metrics.md](09_metrics.md)
- Team, scope & timeline: [07_team.md](07_team.md)

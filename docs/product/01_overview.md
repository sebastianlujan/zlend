# ZLend — Product Overview

## One-Liner

**ZLend lets you borrow stablecoins using your ZCash as collateral — without anyone seeing what you own.**

Your collateral stays private. Your loan is real. The math checks out, but your identity stays yours.

---

## Diagnosis

Two realities collide in crypto today:

1. **DeFi lending is a glass house.** If you borrow on Aave, Compound, or any major lending protocol, your entire financial position is public. How much collateral you have, what your liquidation price is, when you're underwater — all visible to anyone with a block explorer. This isn't a bug. It's how these protocols work. But it means sophisticated actors (MEV bots, liquidation hunters, on-chain analysts) can front-run your positions, target your liquidations, and profile your net worth.

2. **Privacy coins are locked out of DeFi.** ZCash has the strongest privacy technology in crypto — shielded transactions that completely hide sender, receiver, and amount. But ZEC holders can't use their assets productively. There's nowhere to lend or borrow against shielded ZCash. Privacy-conscious holders are stuck: either break their privacy to access DeFi, or keep their ZEC idle.

The result: **people who care most about financial privacy are excluded from the most useful thing in crypto (lending), and people who use lending have zero privacy.**

---

## Guiding Policy

**Build the first lending protocol where privacy is the default, not an add-on.**

We bridge ZCash's proven privacy technology with Avalanche's DeFi infrastructure (Aave V3). Zero-knowledge proofs verify that you have sufficient collateral without revealing what you own or who you are. The user experience should feel like a simple lending app — the cryptography is invisible.

**What this means in practice:**
- Privacy is not optional — it's how the protocol works
- The complexity stays behind the scenes — users see "Lock, Prove, Borrow"
- We build on existing battle-tested infrastructure (Aave V3, ZCash Sapling) rather than reinventing lending or privacy
- We target crypto-curious users who care about privacy but don't want to learn cryptography

**What we're NOT doing:**
- Not building a privacy mixer (we're a lending protocol)
- Not building a DEX or trading platform
- Not building a new L1/L2 chain
- Not requiring users to understand ZK proofs, nullifiers, or key derivation
- Not competing with Aave — we build on top of it

---

## Coherent Actions

1. **Smart contracts on Avalanche** — ZLendContract orchestrates the full borrow/repay lifecycle, verified by Ultrahonk ZK proofs, powered by Aave V3's lending pool
2. **Browser-based key management** — Spending keys never leave the user's device. Proof generation happens locally.
3. **Privacy relayer** — Breaks the on-chain link between your ZCash identity and your Avalanche borrow. Submits transactions on your behalf.
4. **Compliance-ready privacy** — Privacy Pools enable users to prove their collateral source is clean without revealing it. Selective disclosure for regulators.
5. **Phased privacy expansion** — MVP ships with private collateral verification. Later phases add encrypted balances (ZAMA fhEVM) and full transaction privacy.

---

## Vision

**Phase 1 (MVP):** Borrow stablecoins privately using ZCash collateral on Avalanche. Collateral source is hidden. Borrow amounts are public.

**Phase 2:** Encrypted balances via ZAMA fhEVM. Even borrow amounts become private. Full transaction privacy with ZK regex event matching.

**Phase 3:** Multi-asset private collateral. Threshold signatures for shared positions. Privacy Pools integration via Kohaku patterns. ZLend becomes the private lending layer for DeFi.

The end state: **any crypto asset as private collateral, any lending market as the backend, complete financial privacy by default.**

---

## Links

- Technical architecture: [../01_architecture.md](../01_architecture.md)
- Protocol specification: [../02_protocol.md](../02_protocol.md)
- Problem deep-dive: [02_problem.md](02_problem.md)
- Proposed solution: [03_solution.md](03_solution.md)

# Problem Identification

## The Problem in One Sentence

**If you want to borrow against your crypto, everyone can see what you own, how much you borrowed, and when you might get liquidated.**

There is no way to access DeFi lending privately. And if you hold privacy coins like ZCash, you can't access lending at all.

---

## Three Layers of the Problem

### Layer 1: DeFi Lending is a Glass House

Every lending position on Aave, Compound, or MakerDAO is fully public. Anyone with a block explorer can see:

- **Your collateral** — what assets you deposited and how much
- **Your debt** — what you borrowed and at what rate
- **Your liquidation price** — the exact price where your position gets forcibly closed
- **Your wallet history** — every deposit, withdrawal, and repayment

This isn't a side effect. Transparency is baked into how these protocols work. But it creates real problems:

- **MEV bots front-run liquidations.** When your position approaches liquidation, bots race to liquidate you first, extracting maximum value and giving you worse terms.
- **On-chain profiling.** Analytics firms and "whale watchers" build profiles of large holders. Your financial life becomes public data.
- **Targeted attacks.** If someone knows your liquidation price, they can attempt to manipulate the market to trigger it. This has happened on smaller protocols.
- **Tax and legal exposure.** Every transaction is a permanent public record. In jurisdictions with aggressive enforcement, this creates compliance burden and privacy risk.

### Layer 2: Privacy Coins Can't Participate in DeFi

ZCash has the strongest privacy technology in crypto — shielded transactions that completely hide the sender, receiver, and amount using zero-knowledge proofs. But ZEC holders face a choice:

- **Keep privacy, lose opportunity.** Shielded ZEC sits idle. There's no lending, no yield, no borrowing.
- **Break privacy, access DeFi.** Bridge ZEC to a transparent chain, lose all privacy guarantees, and you're back in the glass house.

This isn't unique to ZCash. Monero, Zcash, and other privacy-focused assets are locked out of DeFi entirely.

### Layer 3: Existing Privacy Tools Don't Solve Lending

Current privacy solutions in crypto (Tornado Cash, Railgun, mixers) focus on **transfers** — making it hard to trace where money went. They don't solve the lending problem:

- You can't use a mixer as collateral
- Mixers don't verify solvency — they break the link between transactions
- No existing tool lets you prove "I have enough collateral" without revealing the collateral itself

---

## Problem Validation Score

Using the Problem Validation framework (Frequency x Intensity x Workarounds x WTP):

| Dimension | Score | Evidence |
|-----------|-------|----------|
| **Frequency** | 4/5 (Weekly) | Active DeFi users manage positions weekly. Privacy-conscious holders check markets daily but can't participate. |
| **Intensity** | 4/5 (Major friction) | MEV extraction on DeFi lending is estimated at $100M+/year. Position profiling leads to targeted liquidations. ZEC holders have 5.1M ZEC (~$255M) in shielded pools — growing 5x in two years — with zero DeFi access ([source](https://app.blockworks.co/analytics/network-overview/zcash)). |
| **Workarounds** | 4/5 (Cobbled solutions) | **Evidence needed — currently asserted, not observed.** Plausible workarounds: (1) Multiple wallets to obscure total position — common DeFi practice, no ZCash-specific evidence collected. (2) CeFi lending for pseudo-privacy — BlockFi, Nexo accepted ZEC before collapses; current CeFi ZEC lending options are limited post-2022 collapses. (3) Not borrowing at all — 5.1M ZEC in shielded pools with zero DeFi access is indirect evidence. (4) Wrapping ZEC on other chains — 290K ZEC wrapped is direct evidence of workaround behavior, but wrapping is for DeFi access broadly, not lending specifically. **Must validate with 5+ user interviews before scoring as confirmed 4/5.** |
| **Willingness to Pay** | 3/5 (Probably would pay) | Privacy premium exists — ZCash users already pay higher fees for shielded transactions. Tornado Cash processed $8B+ before sanctions, showing demand for privacy in transfers. **No direct WTP evidence for private lending specifically.** Tornado Cash proves willingness to pay for transfer privacy, not lending privacy — these are different products with different value propositions. |

**Total Score: 4 x 4 x 4 x 3 = 192**

**Assessment: Investigate More (score 100-249).** The problem-validation framework requires 250+ to proceed to "Build." Our score of 192 places us in the "Investigate More" range — the problem is promising but NOT validated sufficiently to commit to building with full confidence.

### What This Score Means

**What is strong (and can be kept):**
- Frequency (4/5): ZCash shielded pool growth (5x in 2 years) and daily transaction activity (400-1,000/day) demonstrate active engagement.
- Intensity (4/5): MEV extraction data ($100M+/year) and $255M locked in shielded pools with zero DeFi access show real friction.

**What is weak (and must be strengthened):**
- Workarounds (4/5): Currently asserted without primary evidence. No user quotes, no observed behavior, no forum thread citations. Plausible but unverified.
- WTP (3/5): No direct evidence that ZCash users would pay for private lending specifically. Adjacent signals exist (shielded tx fees, Tornado Cash volume) but are not the same product category.

### Validation Tasks Required

Per the framework's instruction to "talk to at least 5 people in the ICP":

1. **5+ conversations with ZCash shielded pool users** — Ask: Would you deposit ZEC into a semi-custodial protocol to borrow USDC? What would make you trust it? What rate premium would you accept over transparent lending?
2. **3+ conversations with users who wrapped ZEC on Solana/BSC** — Ask: Why did you sacrifice privacy? Would you have used a privacy-preserving alternative? What did you do with the wrapped ZEC?
3. **Forum/Discord evidence collection** — Search ZCash forums for complaints about DeFi access, requests for lending, discussions of wrapped ZEC dissatisfaction. Cite specific threads.
4. **WTP signal from adjacent products** — Document actual fee premiums paid for privacy (shielded tx fees vs transparent, Railgun usage data, privacy pool fees).

**Target:** Raise the score to 250+ with primary evidence, or explicitly document why the team proceeds despite a sub-250 score (e.g., hackathon context, learning-oriented build, willingness to pivot).

---

## Competitive Alternatives

What do people do today instead of private lending?

| Alternative | What They Do | Limitations |
|-------------|-------------|-------------|
| **Accept transparency** | Borrow on Aave/Compound with full public visibility | No privacy. Positions visible, liquidation risk exposed. |
| **Don't borrow** | Hold crypto without accessing liquidity | Miss opportunities. Can't use capital productively. |
| **Use CeFi** | Borrow on centralized platforms (Nexo, Ledn) | Custodial risk (FTX, Celsius, BlockFi all collapsed). KYC required. Platform sees everything. |
| **Multiple wallets** | Split holdings across wallets to obscure total position | Doesn't hide individual positions. Gas costs multiply. Complex to manage. |
| **OTC desks** | Borrow from private parties via over-the-counter deals | High minimums ($100K+). Counterparty risk. Not accessible to regular users. |
| **Don't use privacy coins** | Sell ZEC for ETH/BTC and use DeFi normally | Defeats the purpose. Loses privacy. Taxable event. |

**Key insight:** There is no option that combines DeFi lending with financial privacy. Users are forced to choose between access and privacy.

---

## Who Feels This Pain Most?

1. **ZCash holders** — Largest unserved group. 5.1M ZEC (~$255M) in shielded pools, growing 5x in two years, with zero lending options. An additional 290K ZEC (~$14.5M) is already wrapped on other chains (Solana, BSC, Near) — proving cross-chain demand exists — but zero ZEC is on Avalanche today ([source](https://app.blockworks.co/analytics/network-overview/zcash)).
2. **Privacy-conscious DeFi users** — Already borrowing but uncomfortable with transparency. Creating workarounds (multiple wallets, timing strategies).
3. **High-net-worth crypto holders** — Don't want position sizes visible. Currently using OTC or CeFi for pseudo-privacy.
4. **Users in high-surveillance jurisdictions** — Financial privacy is a safety concern, not just a preference.

---

## Regulatory Risk Assessment

### Why This Matters

ZCash is under active regulatory scrutiny. Any protocol built on ZCash inherits this risk. Ignoring it is not a product strategy decision — it is a blind spot that could kill the project regardless of product-market fit.

### Known Regulatory Actions Involving Privacy Coins/Protocols

| Event | Date | Relevance |
|-------|------|-----------|
| OFAC sanctions Tornado Cash | Aug 2022 | Privacy-preserving DeFi protocol sanctioned. Smart contract addresses added to SDN list. |
| Tornado Cash developer arrested (Netherlands) | Aug 2022 | Developer liability for privacy protocols established as precedent. |
| Japanese exchanges delist ZCash, Monero, Dash | 2018-2023 | Regulatory pressure leads to reduced ZCash accessibility. |
| EU MiCA regulation | 2024-2025 | Provisions around privacy coins in regulated exchanges. |
| FinCEN proposed rule on convertible virtual currency mixing | 2023 | Explicit targeting of "mixing" and "anonymity-enhanced" transactions. |

### Risk Matrix for ZLend

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| OFAC adds ZLend smart contracts to SDN list | Medium | Fatal | Privacy Pools compliance layer (Phase 2). Selective disclosure via viewing keys. |
| Exchanges refuse USDC deposits from ZLend-connected wallets | Medium | High | Use fresh Avalanche addresses. USDC is not ZEC — harder to flag. |
| Team legal liability (Tornado Cash precedent) | Low-Medium | High | Incorporate in favorable jurisdiction. Open-source decentralization roadmap. |
| ZCash itself faces further delistings | Medium | Medium | Does not affect shielded pool usage, only fiat on/off ramps. |
| Avalanche Foundation distances from ZLend | Low | Medium | Build independently. Do not depend on foundation endorsement. |

### Position Statement

ZLend is NOT a mixer. Key distinctions from Tornado Cash:

1. **Purpose:** Lending (borrowing against collateral), not obfuscating fund flows
2. **Traceability:** On-chain proof of obligation creates an auditable trail
3. **Compliance path:** Viewing keys enable selective disclosure to regulators if required
4. **No pooling of funds:** Each escrow address is per-user, not a shared anonymity pool

These distinctions may or may not satisfy regulators. The team must consult legal counsel before mainnet deployment. Budget for legal review should be part of the funding plan.

---

## Links

- Overview: [01_overview.md](01_overview.md)
- Proposed solution: [03_solution.md](03_solution.md)
- User personas: [05_user-persona.md](05_user-persona.md)
- Market data & analytics: [08_market-data.md](08_market-data.md)

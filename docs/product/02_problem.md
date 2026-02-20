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
| **Intensity** | 4/5 (Major friction) | MEV extraction on DeFi lending is estimated at $100M+/year. Position profiling leads to targeted liquidations. ZEC holders have $500M+ in shielded pools with zero DeFi access. |
| **Workarounds** | 4/5 (Cobbled solutions) | Users create multiple wallets to obscure holdings. Some use CeFi (BlockFi, Nexo) accepting custodial risk for pseudo-privacy. Others simply don't borrow and miss opportunities. |
| **Willingness to Pay** | 3/5 (Probably would pay) | Privacy premium exists — ZCash users already pay higher fees for shielded transactions. Tornado Cash processed $8B+ before sanctions, showing demand. No direct WTP evidence for private lending specifically. |

**Total Score: 4 x 4 x 4 x 3 = 192**

**Assessment: Promising.** Strong evidence on frequency, intensity, and workarounds. WTP needs more validation — specifically, would crypto-curious users pay a premium for private lending over transparent lending?

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

1. **ZCash holders** — Largest unserved group. $500M+ in shielded pools with zero lending options.
2. **Privacy-conscious DeFi users** — Already borrowing but uncomfortable with transparency. Creating workarounds (multiple wallets, timing strategies).
3. **High-net-worth crypto holders** — Don't want position sizes visible. Currently using OTC or CeFi for pseudo-privacy.
4. **Users in high-surveillance jurisdictions** — Financial privacy is a safety concern, not just a preference.

---

## Links

- Overview: [01_overview.md](01_overview.md)
- Proposed solution: [03_solution.md](03_solution.md)
- User personas: [05_user-persona.md](05_user-persona.md)

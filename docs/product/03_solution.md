# Proposed Solution

## The Job To Be Done

> **Phase 1 (Morgan):** "When I need liquidity but my holdings are in shielded ZEC with no DeFi access, I want to borrow stablecoins against my ZEC without breaking my privacy, so I can access funds without selling my position or exposing my financial activity on a public chain."

> **Phase 2 (Alex):** "When I need cash but don't want to sell my crypto, and I see that existing lending platforms make my entire financial position public, I want to borrow against my holdings privately, so I can access liquidity without feeling surveilled."

ZLend is hired for this job. It replaces the current workaround of either accepting transparency, not borrowing, or trusting a centralized platform.

---

## The Solution in Plain Language

**Deposit your ZCash into ZLend. Prove you have collateral. Borrow USDC. Nobody sees what you own.**

Here's what happens:

1. **You request an account** — ZLend creates a new ZCash address for you and gives you a viewing key. The protocol holds the spending key (like a bank holds your safety deposit box key).
2. **You deposit ZCash** — Send your ZEC to the address. Your viewing key lets you see the balance and confirm the deposit arrived.
3. **You prove and borrow** — Your browser generates a mathematical proof that your deposit exists and is sufficient. You submit this proof to the ZLend contract on Avalanche, along with the amount you want to borrow. The contract verifies the proof and borrows USDC from Aave V3 on your behalf.
4. **You repay and claim** — When you're done, repay the USDC on Avalanche. Generate a repayment proof and submit it to the contract. The contract verifies repayment and signals the protocol to release your ZCash back to your origin address.

The zero-knowledge cryptography happens in your browser, automatically. You never see a proof, a nullifier, or a key derivation. You see a lending app.

---

## Product Positioning

Using April Dunford's five-step framework:

### Step 1: Competitive Alternatives

What would our best customers do without ZLend?

| Alternative | Why It Fails |
|-------------|-------------|
| Borrow on Aave/Compound | Zero privacy. Position fully visible. Subject to MEV and liquidation hunting. |
| Don't borrow | Capital sits idle. Can't access liquidity. |
| CeFi lending (Nexo, Ledn) | Custodial risk (FTX, Celsius, BlockFi all collapsed). KYC required. Company sees everything. |
| Multiple wallets | Doesn't actually hide positions. Just adds complexity. |
| Sell ZEC for ETH, borrow normally | Defeats privacy purpose. Taxable event. Gives up shielded holding. |

### Step 2: Unique Attributes

What does ZLend have that alternatives don't?

| Attribute | Verifiable? |
|-----------|-------------|
| **Cross-chain private collateral** — ZCash used as collateral on Avalanche without revealing balances | Yes — first protocol to do this |
| **ZK-verified solvency** — Collateral proven via zero-knowledge proofs, not by revealing balances | Yes — Ultrahonk proofs on-chain |
| **Escrow-based custody** — Protocol holds ZCash during loan period with on-chain proof of obligation to return it | Yes — verifiable on both chains |
| **Identity-collateral unlinkability** — Nobody can connect your ZCash deposits to your Avalanche borrow | Yes — protocol acts as intermediary |
| **Built on Aave V3** — Uses battle-tested lending infrastructure, not a new untested pool | Yes — existing Avalanche deployment |

### Step 3: Value

What outcomes do these attributes enable?

- **Borrow without financial exposure** — Access liquidity without the world seeing your position
- **Use privacy coins productively** — ZEC finally works in DeFi, without breaking privacy
- **Protection from MEV** — Hidden positions can't be front-run or targeted for liquidation
- **On-chain accountability** — The repayment proof creates a verifiable obligation for the protocol to return your ZCash
- **Simplicity** — It looks like a regular lending app. The ZK cryptography is invisible.

### Step 4: Best-Fit Customers

Who gets the most value from these attributes? We target two personas in sequence — see [User Personas](05_user-persona.md) for the full growth strategy.

**Launch (Phase 1):** Existing ZCash holders locked out of DeFi. They understand privacy deeply and have been waiting for this. 5.1M ZEC (~$255M) sits in shielded pools today, growing 5x in two years, with 400-1,000 daily private transactions showing active engagement ([source](https://app.blockworks.co/analytics/network-overview/zcash)). These users validate the protocol, stress-test the cryptography, and become the trust signal for Phase 2.

**Proven demand:** 290K ZEC (~$14.5M) is already wrapped on other chains (Solana ~150K, BSC ~110K, Near ~20K) — ZCash holders sacrificing their privacy just to access DeFi. **Zero ZEC exists on Avalanche today.** ZLend gives them a path that doesn't require that sacrifice.

**Growth (Phase 2):** Crypto-curious holders who value privacy but aren't DeFi-native. They have ZCash (or would buy it) and want to borrow without exposing themselves. They arrive when social proof from Phase 1 exists — "used by the ZCash community" is the trust signal they need.

**Not our customer:** Day traders, MEV operators, institutions needing full audit trails, users who don't care about privacy.

### Step 5: Market Category

**Private DeFi Lending** — a new subcategory of DeFi lending.

We're not competing with Aave (we build on it). We're not a mixer (we're a lending protocol). We're creating the category where lending and privacy intersect.

### Category Creation Risk

April Dunford warns: "Creating a new category is almost always the wrong choice for startups." It requires educating the market on what the category is before you can sell your product. A two-person team rarely has the resources for this.

**Our mitigation:** We do not need to establish "Private DeFi Lending" as a recognized category to succeed in Phase 1. Morgan (ZCash native) already understands the problem — they have been asking for it. The category label is useful for positioning documents and grant applications, not for user acquisition.

**The risk materializes in Phase 2** when targeting Alex, who does NOT know this category exists. At that point, we need either:
1. Enough Phase 1 traction that the product speaks for itself ("borrow against ZEC privately" is a feature, not a category)
2. Or a repositioning: instead of "Private DeFi Lending" (new category), position as "DeFi Lending" (existing category) with "privacy-preserving" as a differentiating attribute

**Recommended Phase 1 positioning:** "DeFi Lending for ZCash" (existing category: DeFi Lending; differentiator: accepts ZCash as collateral with privacy preserved). Defer category creation until Phase 2 resources and market validation exist.

---

## Forces of Progress

See persona-specific Forces of Progress analysis in [05_user-persona.md](05_user-persona.md):
- [Morgan (Phase 1)](05_user-persona.md#forces-of-progress) — Driving forces and resistance specific to ZCash-native users
- [Alex (Phase 2)](05_user-persona.md#forces-of-progress-1) — Driving forces and resistance specific to crypto-curious holders

The persona-specific versions are the canonical analysis. They differ meaningfully: Morgan's anxiety centers on cryptographic security ("Is Ultrahonk secure?"), while Alex's centers on custodial trust ("What if they don't return my ZEC?"). A single generic Forces of Progress analysis loses this nuance.

---

## What We're NOT Building

| Not This | Why Not |
|----------|---------|
| A privacy mixer | We're a lending protocol. Privacy is the mechanism, not the product. |
| A DEX or trading platform | We do one thing: private lending. |
| A new blockchain or L2 | We deploy on existing Avalanche infrastructure. |
| A privacy wallet | We complement wallets, not replace them. |
| An Aave competitor | We build on Aave V3. They're infrastructure, we're the privacy layer. |
| A fully non-custodial protocol | ZLend holds your ZCash in escrow during the loan. This is an honest trade-off for simplicity and cross-chain functionality. |

---

## Links

- Problem deep-dive: [02_problem.md](02_problem.md)
- Product architecture: [04_architecture.md](04_architecture.md)
- User personas: [05_user-persona.md](05_user-persona.md)
- Market data & analytics: [08_market-data.md](08_market-data.md)
- Technical protocol spec: [../02_protocol.md](../02_protocol.md)

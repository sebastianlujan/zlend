# Proposed Solution

## The Job To Be Done

> **Phase 1 (Morgan):** "When I have ZEC with no DeFi access, I want to use it as collateral to borrow stablecoins on Avalanche, so my capital works for me — without giving up my privacy."

> **Phase 2 (Alex):** "When I need cash but don't want to sell my crypto, I want to borrow against my holdings on Avalanche without revealing my financial position, so I can access liquidity while keeping my position and my privacy."

OGBank is hired for this job. It replaces the current workaround of either wrapping ZEC (sacrificing privacy), not borrowing, or trusting a centralized platform.

---

## The Solution in Plain Language

**Deposit your ZCash into OGBank. Prove you have collateral. Borrow USDC on Avalanche. Your privacy stays intact.**

Here's what happens:

1. **You request an account** — OGBank creates a new ZCash address for you and gives you a viewing key. The protocol holds the spending key (like a bank holds your safety deposit box key).
2. **You deposit ZCash** — Send your ZEC to the address. Your viewing key lets you see the balance and confirm the deposit arrived.
3. **You prove and borrow** — Your browser generates a mathematical proof that your deposit exists and is sufficient. You submit this proof to the OGBank contract on Avalanche, along with the amount you want to borrow. The contract verifies the proof and borrows USDC from Aave V3 on your behalf.
4. **You repay and claim** — When you're done, repay the USDC on Avalanche. Generate a repayment proof and submit it to the contract. The contract verifies repayment and signals the protocol to release your ZCash back to your origin address.

The zero-knowledge cryptography happens in your browser, automatically. You never see a proof, a nullifier, or a key derivation. You see a lending app.

---

## Product Positioning

Using April Dunford's five-step framework:

### Step 1: Competitive Alternatives

What would our best customers do without OGBank?

| Alternative | Why It Fails |
|-------------|-------------|
| Wrap ZEC on Solana/BSC | Loses privacy — the core value of holding ZEC. 290K already wrapped, proving demand but breaking the value prop. |
| Don't borrow | Capital sits idle. $255M in ZEC not working. Can't access liquidity. |
| Borrow on Aave/Compound | Can't use ZEC. Zero privacy. Position fully visible. Subject to MEV. |
| CeFi lending (Nexo, Ledn) | Custodial risk (FTX, Celsius, BlockFi all collapsed). KYC required. Company sees everything. |
| Sell ZEC for ETH, borrow normally | Defeats the purpose. Loses privacy. Taxable event. Gives up shielded holding. |

### Step 2: Unique Attributes

What does OGBank have that alternatives don't?

| Attribute | Verifiable? |
|-----------|-------------|
| **ZEC-to-Avalanche DeFi gateway** — First protocol to bring ZCash capital to Avalanche's DeFi ecosystem | Yes — zero ZEC on Avalanche today |
| **Cross-chain private collateral** — ZCash used as collateral on Avalanche without revealing balances | Yes — first protocol to do this |
| **ZK-verified solvency** — Collateral proven via zero-knowledge proofs, not by revealing balances | Yes — Ultrahonk proofs on-chain |
| **Escrow-based custody** — Protocol holds ZCash during loan period with on-chain proof of obligation to return it | Yes — verifiable on both chains |
| **Built on Aave V3** — Uses battle-tested lending infrastructure, not a new untested pool | Yes — existing Avalanche deployment |

### Step 3: Value

What outcomes do these attributes enable?

- **Unlock ZEC capital in DeFi** — $255M in ZEC finally has a productive use, and it flows to Avalanche
- **New TVL for Avalanche** — Every ZEC deposit brings capital that doesn't exist on Avalanche today
- **Use privacy coins productively** — ZEC works in DeFi without breaking privacy
- **Protection from MEV** — Hidden positions can't be front-run or targeted for liquidation
- **On-chain accountability** — The repayment proof creates a verifiable obligation for the protocol to return your ZCash
- **Simplicity** — It looks like a regular lending app. The ZK cryptography is invisible.

### Step 4: Best-Fit Customers

Who gets the most value from these attributes? We target two personas in sequence — see [User Personas](05_user-persona.md) for the full growth strategy.

**Launch (Phase 1):** Existing ZCash holders locked out of DeFi. They understand privacy deeply and have been waiting for DeFi access. 5.1M ZEC (~$255M) sits in shielded pools today, growing 5x in two years, with 400-1,000 daily private transactions showing active engagement ([source](https://app.blockworks.co/analytics/network-overview/zcash)). These users validate the protocol, stress-test the cryptography, and become the trust signal for Phase 2.

**Proven demand:** 290K ZEC (~$14.5M) is already wrapped on other chains (Solana ~150K, BSC ~110K, Near ~20K) — ZCash holders sacrificing their privacy just to access DeFi. **Zero ZEC exists on Avalanche today.** OGBank gives them a path that doesn't require that sacrifice — and routes their capital through Avalanche.

**Growth (Phase 2):** Crypto-curious holders who value privacy but aren't DeFi-native. They have ZCash (or would buy it) and want to borrow without exposing themselves. They arrive when social proof from Phase 1 exists — "used by the ZCash community" is the trust signal they need.

**Not our customer:** Day traders, MEV operators, institutions needing full audit trails, users who don't care about privacy.

### Step 5: Market Category

**DeFi Gateway for ZCash on Avalanche** — positioned within the existing DeFi Lending category.

We're not competing with Aave (we build on it and feed it new borrowing volume). We're not a mixer (we're a lending protocol). We're the first gateway that routes ZCash capital into Avalanche's DeFi ecosystem, with privacy preserved as a core feature.

### Why This Positioning Works

Unlike "Private DeFi Lending" (a new category that requires market education), "DeFi Gateway for ZCash" fits within existing mental models:

1. **No category creation needed** — Morgan (ZCash native) already understands the problem. "Your ZEC can finally work in DeFi" is a feature, not a new category.
2. **Clear value to Avalanche** — We bring new capital and new users. This resonates with ecosystem grants and partnerships.
3. **Privacy as differentiator, not category** — Privacy is what makes OGBank better than wrapping ZEC, not what defines the product category.

**Phase 2 consideration:** When targeting Alex (crypto-curious), the positioning shifts to "Borrow against your crypto on Avalanche" with privacy as a feature highlight, not the headline.

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
| A privacy mixer | We're a lending protocol. Privacy is a feature, not the product. |
| A DEX or trading platform | We do one thing: bring ZEC to Avalanche DeFi via lending. |
| A new blockchain or L2 | We deploy on existing Avalanche infrastructure. |
| A privacy wallet | We complement wallets, not replace them. |
| An Aave competitor | We build on Aave V3. They're infrastructure, we're the ZEC gateway. |
| A fully non-custodial protocol | OGBank holds your ZCash in escrow during the loan. This is an honest trade-off for simplicity and cross-chain functionality. |

---

## Links

- Problem deep-dive: [02_problem.md](02_problem.md)
- Product architecture: [04_architecture.md](04_architecture.md)
- User personas: [05_user-persona.md](05_user-persona.md)
- Market data & analytics: [08_market-data.md](08_market-data.md)
- Technical protocol spec: [../02_protocol.md](../02_protocol.md)

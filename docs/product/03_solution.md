# Proposed Solution

## The Job To Be Done

> **"When I want to borrow against my crypto holdings, I want to do it privately, so I can access liquidity without exposing my financial position to the world."**

ZLend is hired for this job. It replaces the current workaround of either accepting transparency, not borrowing, or trusting a centralized platform.

---

## The Solution in Plain Language

**Lock your ZCash as collateral. Borrow stablecoins. Nobody sees what you own.**

Here's what happens:

1. **You lock ZCash** — Your shielded ZEC goes into a private vault. It stays encrypted.
2. **ZLend proves you're solvent** — A mathematical proof verifies that your collateral is sufficient. It says "yes, they have enough" without revealing how much, where it came from, or who you are.
3. **You receive stablecoins** — Borrowed tokens arrive in your Avalanche wallet. You can use them however you want.
4. **You repay and unlock** — When you're done, repay the loan. Another proof confirms the repayment, and your ZCash is released.

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
| **Cross-chain private collateral** — ZCash shielded UTXOs used as collateral on Avalanche | Yes — first protocol to do this |
| **ZK-verified solvency** — Collateral proven via zero-knowledge proofs, not by revealing balances | Yes — Ultrahonk proofs on-chain |
| **Non-custodial** — Spending keys never leave the user's browser | Yes — verifiable client-side architecture |
| **Identity-collateral unlinkability** — Nobody can connect your ZCash address to your Avalanche borrow | Yes — relayer breaks the link |
| **Built on Aave V3** — Uses battle-tested lending infrastructure, not a new untested pool | Yes — existing Avalanche deployment |

### Step 3: Value

What outcomes do these attributes enable?

- **Borrow without financial exposure** — Access liquidity without the world seeing your position
- **Use privacy coins productively** — ZEC finally works in DeFi, without breaking privacy
- **Protection from MEV** — Hidden positions can't be front-run or targeted for liquidation
- **Non-custodial security** — No FTX/Celsius risk. You hold your keys.
- **Compliance-compatible privacy** — Prove your collateral is clean without revealing it (Privacy Pools)

### Step 4: Best-Fit Customers

Who gets the most value from these attributes?

**Primary:** Crypto-curious holders who value privacy. They have ZCash (or would buy it) and want to borrow without exposing themselves. Not DeFi power users — they want simplicity.

**Secondary:** Existing ZCash holders who have been locked out of DeFi entirely. They understand privacy deeply and have been waiting for this.

**Not our customer (yet):** Day traders, MEV operators, institutions needing full audit trails, users who don't care about privacy.

### Step 5: Market Category

**Private DeFi Lending** — a new subcategory of DeFi lending.

We're not competing with Aave (we build on it). We're not a mixer (we're a lending protocol). We're creating the category where lending and privacy intersect.

---

## Forces of Progress

What drives adoption and what resists it:

### Driving Adoption

| Force | Description |
|-------|-------------|
| **Push** (pain with current) | "My Aave position is visible to everyone. A whale watcher bot tweeted my liquidation level. MEV bots front-ran my repayment." |
| **Pull** (attraction to new) | "I can borrow without anyone knowing my position. My ZCash finally earns something. The app looks simple." |

### Resisting Adoption

| Force | Description |
|-------|-------------|
| **Anxiety** (fear of new) | "Is the ZK math actually secure? What if the relayer steals my collateral? What if the protocol gets hacked? Is this regulatory-safe?" |
| **Habit** (comfort with old) | "I already know how Aave works. My ZEC is fine sitting in my wallet. CeFi is easier even if riskier." |

### How We Win

**Push + Pull must be stronger than Anxiety + Habit.**

- We reduce **Anxiety** by: building on Aave V3 (trusted infrastructure), non-custodial design (relayer can't steal), ZK proofs are mathematically verifiable, Privacy Pools for compliance
- We break **Habit** by: making the UX as simple as existing lending apps, requiring zero ZK knowledge, providing clear yield opportunity for idle ZEC

---

## What We're NOT Building

| Not This | Why Not |
|----------|---------|
| A privacy mixer | We're a lending protocol. Privacy is the mechanism, not the product. |
| A DEX or trading platform | We do one thing: private lending. |
| A new blockchain or L2 | We deploy on existing Avalanche infrastructure. |
| A privacy wallet | We complement wallets, not replace them. |
| An Aave competitor | We build on Aave V3. They're infrastructure, we're the privacy layer. |

---

## Links

- Problem deep-dive: [02_problem.md](02_problem.md)
- Product architecture: [04_architecture.md](04_architecture.md)
- User personas: [05_user-persona.md](05_user-persona.md)
- Technical protocol spec: [../02_protocol.md](../02_protocol.md)

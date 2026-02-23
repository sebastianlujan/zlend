# Market Data & Avalanche Value Proposition

## Data Source

All analytics from [Blockworks ZCash Dashboard](https://app.blockworks.co/analytics/network-overview/zcash) — accessed February 2026.

---

## ZCash Shielded Supply

**5.1M ZEC in shielded pools.** Up from ~1M in early 2024 — a **5x increase in under two years.**

The growth is parabolic, accelerating sharply from July 2025 onward. This isn't gradual adoption — it's a trend shift. Users are actively choosing privacy.

At current ZEC prices (~$40-50), this represents **~$200-255M in private capital with zero DeFi access.**

### By Pool

| Pool | Supply | Trend |
|------|--------|-------|
| **Orchard** | ~4M+ ZEC | Explosive growth — from near 0 to dominant in under 2 years |
| **Sapling** | ~1M ZEC | Flat, stable baseline |

Orchard is ZCash's latest privacy upgrade. The fact that almost all new shielded deposits go to Orchard signals that ZCash users are technically engaged — they upgrade to the best available privacy. This is the kind of user who would adopt OGBank.

---

## Transaction Activity

### Shielded Transactions (Weekly)

| Metric | Value |
|--------|-------|
| Current | ~10-12K/week |
| Peak (May 2024) | ~24K/week |
| Peak (Oct 2025) | ~19K/week |

Consistent activity with spikes during market volatility — exactly when users want to borrow (access liquidity without selling).

### Private Transactions (Daily)

| Type | Volume |
|------|--------|
| Total private txs | ~400-1,000/day |
| Dominant type | Orchard Internal (z→z, fully shielded) |
| Sapling baseline | ~100/day |

"Orchard Internal" means fully shielded transactions where both sender and receiver are hidden. These users are the most privacy-committed — OGBank's launch persona (Morgan, the ZCash Native).

---

## Transfer Volume

**$100M-$500M daily**, with spikes up to **$1.4B**.

Mix of transparent (dominant) and shielded volume. The shielded portion (Orchard) is growing.

**Important caveat:** Transfer volume measures ZEC moving between addresses. It does NOT directly indicate lending demand. A user transferring ZEC is performing a different action (payment, exchange deposit, self-transfer) than a user depositing ZEC as collateral for a loan. We cannot extrapolate collateral demand from transfer volume.

**More relevant demand signals:**
- **290K ZEC wrapped on other chains:** Directly demonstrates willingness to lock up ZEC in a cross-chain protocol. These users took an action (wrapping) analogous to depositing collateral.
- **5.1M ZEC in shielded pools:** Demonstrates long-term holding behavior. These users are NOT spending their ZEC — they are storing it. Borrowing against stored assets is a natural product extension.
- **Zero ZEC DeFi lending exists today:** We cannot measure demand for a product that does not exist. This is why user interviews (see [02_problem.md](02_problem.md#validation-tasks-required)) are essential before committing to build.

---

## Network Revenue (REV)

| Period | REV |
|--------|-----|
| Through 2024 | Near $0 |
| Feb 2025 | First meaningful revenue |
| Oct 2025 peak | ~$130K/week |
| Current | ~$10-20K/week |

ZCash transitioned from zero economic value to measurable revenue. The network is maturing — it's not just a speculative asset. Revenue generation signals sustainability, which matters for collateral reliability.

---

## Wrapped ZEC on Other Chains

This is the most important dataset for OGBank's positioning on Avalanche.

| Chain | Wrapped ZEC | Estimated Value |
|-------|-------------|-----------------|
| Solana | ~150K ZEC | ~$6-7.5M |
| BSC | ~110K ZEC | ~$4.4-5.5M |
| Near | ~20K ZEC | ~$800K-1M |
| Ethereum | Tiny | Negligible |
| **Avalanche** | **0** | **$0** |
| **Total** | **~290K ZEC** | **~$14.5M** |

**Key takeaway:** ZEC holders ARE willing to bridge to other chains — 290K ZEC is already cross-chain. But **Avalanche has captured zero of this demand.** OGBank would be the first protocol to bring ZCash capital to Avalanche.

---

## CEX vs DEX Volume

| Type | Daily Volume |
|------|-------------|
| CEX | ~$100-200M (current), peaked $1.3B (Nov 2025) |
| DEX | Negligible |

ZEC trading is almost 100% centralized. DeFi has not captured ZEC users yet. This is both the challenge (habit of using CEXs) and the opportunity — OGBank doesn't require users to trade on DEXs. It meets them where they are: hold ZEC, borrow USDC.

---

## Market Sizing for OGBank

### Addressable Market

Using wrapped ZEC on other chains as the primary benchmark (wrapping is the closest existing analog to depositing collateral):

| Scenario | ZEC Captured | Estimated TVL | Basis |
|----------|-------------|---------------|-------|
| Conservative | ~20K ZEC | ~$1M | Near-level wrapping (smallest chain) |
| Moderate | ~110K ZEC | ~$5M | BSC-level wrapping |
| Aggressive | ~150K ZEC | ~$7.5M | Solana-level wrapping |
| Theoretical ceiling | 5.1M ZEC | ~$255M | All shielded supply (unrealistic) |

**Key assumption:** Wrapping ZEC (which breaks privacy) approximates depositing ZEC into OGBank (which preserves privacy). If OGBank's privacy preservation genuinely matters to users, actual demand could exceed wrapping benchmarks because users are NOT sacrificing privacy. If the custodial escrow model is a larger barrier than privacy loss, actual demand could be lower.

**Honest assessment:** We do not know which effect dominates. User interviews with wrapped-ZEC users would directly answer this question.

### Growth Trajectory

The 5x shielded supply growth means OGBank's addressable market grows automatically. If shielded supply continues at the current trajectory, the market could reach 8-10M ZEC by end of 2026.

---

## What OGBank Brings to Avalanche

### 1. New Capital Category

Avalanche currently has **zero ZCash capital.** OGBank brings an entirely new asset class to the ecosystem. No other Avalanche protocol connects to ZCash.

### 2. Quantifiable TVL Contribution

Every ZEC deposited into OGBank becomes collateral on Avalanche. Conservative estimate: $1-5M new TVL. Moderate: $5-7.5M. This is TVL that doesn't exist on Avalanche today — and can't come from any other protocol.

### 3. Aave V3 Utilization Boost

Every OGBank borrow increases Aave V3 utilization on Avalanche. OGBank doesn't compete with Aave — it feeds it. More borrowing = more interest for Aave suppliers = more liquidity attracted to Avalanche's Aave deployment.

### 4. Privacy DeFi First Mover

Avalanche becomes the **first major EVM chain with a privacy-preserving lending protocol.** This is a category differentiator vs Ethereum, Solana, BSC, Arbitrum — none of them have this.

### 5. ZCash Community Pipeline

OGBank creates a direct pipeline from ZCash's active user base (~400-1,000 daily privacy transactions) into the Avalanche ecosystem. These users currently have **zero reason** to use Avalanche. OGBank gives them one.

### 6. Growing Market — No Extra Work for Avalanche

The 5x shielded supply growth means OGBank's opportunity grows automatically as ZCash adoption increases. Avalanche benefits from ZCash's momentum without having to drive it.

### 7. Wrapped ZEC Capture

290K ZEC is already wrapped on other chains. None is on Avalanche. OGBank gives ZEC holders a reason to choose Avalanche over Solana, BSC, or Near — not just wrapping, but productive use (borrowing).

---

## Key Numbers Summary

| Metric | Value | Trend |
|--------|-------|-------|
| Shielded ZEC supply | 5.1M ZEC (~$255M) | 5x growth in 2 years |
| Orchard pool share | ~4M+ ZEC (~80%) | Parabolic growth |
| Weekly shielded txs | 10-12K | Stable with spikes |
| Daily private txs | 400-1,000 | Orchard-dominant |
| Daily transfer volume | $100M-$500M | Active |
| Network REV | $10-20K/week | Growing from zero |
| ZEC on other chains | 290K (~$14.5M) | Cross-chain demand proven |
| ZEC on Avalanche | **0** | OGBank changes this |
| CEX/DEX volume ratio | ~99% CEX | DeFi opportunity untapped |

---

## Links

- Product overview: [01_overview.md](01_overview.md)
- Problem validation: [02_problem.md](02_problem.md)
- Solution positioning: [03_solution.md](03_solution.md)
- Data source: [Blockworks ZCash Analytics](https://app.blockworks.co/analytics/network-overview/zcash)

# Metrics Framework

## North Star Metric

**Total ZEC Deposited in Active Escrow (TVL in ZEC terms)**

Why this metric:
- Directly measures the core value proposition (ZCash used productively in DeFi)
- Increases only when real users deposit real collateral
- Not gameable by wash trading or fake accounts (requires actual ZEC in escrow)
- Meaningful to all stakeholders: users (protocol is trusted), team (product works), partners (Avalanche gets new TVL), investors (protocol has traction)

Expressed as: **"X ZEC in active escrow across Y positions"**

**Test:** If this metric goes up, does OGBank sustainably grow? Yes — more ZEC in escrow means more users trust the protocol, more USDC is borrowed from Aave V3, and the protocol's core mechanism is validated.

---

## Input Metrics (What Drives the North Star)

| Metric | Definition | Target (Phase 1) |
|--------|-----------|-------------------|
| **Onboarding completion rate** | % of account-creation starts that result in a confirmed ZEC deposit | > 60% |
| **Average deposit size** | Mean ZEC per escrow position | > 50 ZEC (~$2,500) |
| **Borrow completion rate** | % of deposits that result in a USDC borrow | > 80% |
| **Time to first borrow** | Minutes from deposit confirmation to USDC received | < 10 min |
| **Successful claim rate** | % of repaid loans where ZEC is returned successfully | 100% (anything less is a protocol failure) |

### Input/Output Tree

```
North Star: Total ZEC in Active Escrow
  |
  +-- Acquisition: New ZCash-native users discovering OGBank
  |     +-- ZCash forum/Discord reach
  |     +-- Referrals from existing users
  |
  +-- Activation: % who deposit ZEC after creating an account
  |     +-- Onboarding completion rate
  |     +-- Time from account creation to first deposit
  |
  +-- Engagement: Active borrow positions
  |     +-- Borrow completion rate (deposit -> borrow)
  |     +-- Average borrow duration
  |     +-- Repeat borrow rate (completed cycle -> new deposit)
  |
  +-- Retention: Users who complete full cycle and return
        +-- Successful claim rate
        +-- Repeat usage rate (% who start a second borrow cycle)
        +-- Net Promoter within ZCash community
```

---

## Counter-Metrics (What We Must NOT Sacrifice)

Every metric gets a counter-metric. This prevents gaming and ensures we're not optimizing one thing at the expense of another.

| Metric | Counter-Metric | Why |
|--------|---------------|-----|
| Total ZEC in escrow (TVL) | Number of unique depositors | Prevents "one whale = success" illusion |
| Onboarding completion rate | Time to first deposit | Don't simplify onboarding by hiding critical trust information |
| Borrow completion rate | Proof generation failure rate (< 5%) | Don't push users to borrow if proof system is unreliable |
| Average deposit size | Successful claim rate (100%) | Don't encourage large deposits if claim flow isn't battle-tested |
| Time to first borrow | Mean time to ZEC return (< 5 min) | Don't optimize borrow speed if the return leg is slow |

---

## Guardrail Metrics (Hard Constraints)

| Guardrail | Threshold | If Breached |
|-----------|-----------|-------------|
| **Collateral loss events** | 0 (existential) | Stop all new deposits. Root cause analysis. Public post-mortem. |
| **Claim failure rate** | 0% | Halt new borrows until resolved. Every repaid loan MUST return ZEC. |
| **Proof generation failure rate** | < 5% | Investigate browser compatibility, circuit issues. Do not launch Phase 2 UX if proof system is unreliable. |
| **Relayer uptime** | > 99.5% | ZEC returns depend on relayer. Downtime = user anxiety = trust destroyed. |

---

## ZCash Community Sentiment (Qualitative)

Not everything is a number. For a protocol launching to a tight-knit community, sentiment matters:

| Signal | Positive | Negative |
|--------|----------|----------|
| ZCash Forum posts | Recommending OGBank, reporting successful cycles | Warning against OGBank, reporting issues, custodial risk concerns |
| Discord mentions | Organic questions, integration requests | Silence (worse than complaints — means irrelevance) |
| Developer engagement | Code reviews, circuit audits from community, PRs | No engagement with open-source code |
| Morgan-type endorsement | Known ZCash community members publicly using OGBank | Known members publicly advising against it |

---

## Phase 1 Success Criteria

| Milestone | Definition | Timeframe |
|-----------|-----------|-----------|
| **Proof of Life** | 10 deposits from 10 unique ZCash addresses, all completing full borrow-repay-claim cycle | Month 1 after testnet launch |
| **Early Traction** | 50 deposits, 30+ completed borrow-repay-claim cycles, 0 collateral loss events | Month 3 |
| **Product-Market Fit Signal** | 100 deposits, 50+ unique users, 20%+ repeat usage rate, positive ZCash community sentiment | Month 6 |
| **Phase 2 Gate** | All Phase 1 criteria met + 0 collateral loss events + completed security audit + positive community sentiment | Before investing in Alex-targeted UX |

**Phase 2 investment is gated on Phase 1 success.** The team does NOT invest in UX polish, educational content, SEO, or non-technical onboarding until the Phase 2 Gate is met. This prevents premature investment in features for a user (Alex) who won't arrive until Morgan has validated the protocol.

---

## What We Do NOT Track (and Why)

| Metric | Why We Skip It |
|--------|---------------|
| Total signups / account creations | Vanity. Only matters if they deposit. |
| Page views / website traffic | Vanity. Phase 1 is community-driven, not SEO-driven. |
| Social media followers | Vanity. Morgan doesn't care about Twitter presence. |
| TVL in USD terms | ZEC price volatility distorts this. Track ZEC deposited instead. |
| Number of features shipped | Output metric. Means nothing without user adoption. |

---

## Links

- User personas (who we measure for): [05_user-persona.md](05_user-persona.md)
- User journey (where metrics apply): [06_user-journey.md](06_user-journey.md)
- Team scope and timeline: [07_team.md](07_team.md)
- Product overview: [01_overview.md](01_overview.md)

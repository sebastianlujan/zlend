# Landing Page

**Target persona**: Morgan — The ZCash Native ([05_user-persona.md](05_user-persona.md), Phase 1)
**Functional JTBD**: "I have ZEC, I want DeFi access, I don't want to give up my privacy"
**Overall sentiment**: discovery, curiosity, progressive trust. Never fear, never negativity.

The emotional journey is: "OK, I have capital → others are earning with theirs → I want to keep my privacy → what can I do?" — all from curiosity, not from panic.

---

## Sections

### 1. Hero

**Sentiment**: intrigue, immediate clarity.

| | |
|---|---|
| Headline | Private Lending on Avalanche |
| Subheadline | Borrow USDC using your ZCash as collateral — without anyone seeing what you own. |
| CTA | Learn More → #problem |
| Secondary | Read the Docs → #cta |

**What it achieves**: Value proposition in one sentence. Morgan understands in 3 seconds what ZLend is. The 3D scene (particles + wireframe icosahedron) communicates tech without words.

**Visual**: Full-screen, 3D scene background (Three.js), animated entrance, gradient overlay. Dark.

---

### 2. Problem (The Landscape)

**Sentiment**: discovery, realization. "Oh right — that's how it works. And my ZEC isn't part of it."

| | |
|---|---|
| Label | The Landscape |
| Title | DeFi Is Growing. Where Does Your ZEC Fit? |
| Subtitle | You hold ZEC because privacy matters to you. Meanwhile, DeFi lending keeps expanding. What would it look like to participate — without giving that up? |

**Scene 1 — "How DeFi Lending Works Today"**
A mock DeFi position (wallet, collateral, debt, health, liquidation) in a styled card. Not a threat — an observation: "this is how it works, everything is transparent." Note: "Every position is fully transparent." Three factual observations below.

**Scene 2 — "Meanwhile, 5.1M ZEC Sits Idle"**
Side-by-side comparison: ETH (4.2% APY, lending, borrowing, yield) vs ZEC (0.00% APY, no lending yet, no borrowing yet, no yield yet). The word "yet" is key — it implies possibility, not loss. Closes with a question: "What if your ZEC could work in DeFi — without compromising what makes it valuable?"

**What it achieves**: Morgan sees the landscape, recognizes their reality, and wonders "what can I do?" The closing question leads naturally into Solution.

**Key design decisions**:
- No red colors or warnings — neutral, observational
- No fear-based language (no MEV bots, no whale watchers, no "your financial life is public data")
- The ZEC side doesn't have reduced opacity or a "sad" look — it has dormant potential
- "No lending yet" instead of "No lending" — the "yet" changes everything

---

### 3. Solution

**Sentiment**: clarity, simplicity. "Oh, it's just 3 steps."

| | |
|---|---|
| Label | The Solution |
| Title | ZLend: Where Privacy Meets DeFi |
| Subtitle | Use zero-knowledge proofs to verify your ZCash collateral without revealing what you own. |

**3 steps**:
1. **Deposit** — Send ZEC, get viewing key
2. **Prove & Borrow** — Browser generates ZK proof, borrow USDC from Aave V3
3. **Repay & Claim** — Repay USDC, prove repayment, get ZEC back

**What it achieves**: Reduces complexity to 3 actions. Morgan — being technical — appreciates the simplicity. Doesn't oversimplify, but doesn't overwhelm.

**Visual**: 3 columns with large step numbers (01, 02, 03) in semi-transparent monospace.

---

### 4. How It Works

**Sentiment**: technical confidence. "I understand the mechanics."

| | |
|---|---|
| Label | How It Works |
| Title | Zero-Knowledge Verified Lending |

**6 features**:
1. Cross-Chain Private Collateral
2. ZK-Verified Solvency (Ultrahonk)
3. Built on Aave V3
4. MEV Protection
5. Identity Unlinkability
6. Escrow-Based Custody

**What it achieves**: Technical deep dive for Morgan. Each feature addresses a different concern: privacy, security, integration, custody. Morgan needs to understand the "how" before they trust.

**Visual**: Card grid (2-3 columns), semi-transparent background.

---

### 5. Features (Technical Foundation)

**Sentiment**: credibility. "They're using proven technology."

| | |
|---|---|
| Label | Technical Foundation |
| Title | Built on Proven Primitives |

**4 features with tags**:
1. Noir + Ultrahonk (ZK Proofs)
2. Aave V3 Integration (Infrastructure)
3. ZIP-32 Key Derivation (Cryptography)
4. Nullifier Protection (Security)

**What it achieves**: Name-dropping technologies Morgan knows (Noir, Ultrahonk/Barretenberg, ZIP-32, Aave V3). If you know these words, you trust. If you don't, the fact that they exist and have names gives credibility.

**Visual**: 2 columns, monospace tags in primary color, hover effects.

---

### 6. Market Data

**Sentiment**: validation. "There's real opportunity here."

| | |
|---|---|
| Label | Market Opportunity |
| Title | Untapped Capital, Waiting for Access |

**4 stats**:
| Stat | Context |
|------|---------|
| 5.1M ZEC | In shielded pools (~$255M, up 5x in 2 years) |
| $0 | DeFi access for shielded ZEC — ZLend changes this |
| 100% | ZEC stays on ZCash — no wrapping, no synthetic tokens |
| 10-12K | Weekly shielded transactions — active users |

**Source**: Blockworks Analytics (with link).

**What it achieves**: Real data proving the opportunity exists. Morgan knows these numbers — seeing them on the landing validates that the team did their research.

**Visual**: StatCard grid (4 columns), monospace for numbers.

---

### 7. Trust

**Sentiment**: transparency, honesty. "They're honest about the trade-offs."

| | |
|---|---|
| Label | Trust Model |
| Title | Transparent About Trade-Offs |

**Custody model**:
- **Viewing Key** (you hold this) — verify balance, cannot move funds
- **Spending Key** (protocol holds this) — move ZEC in/out of escrow

**Privacy table**:
| Data | Who can see it |
|------|----------------|
| Your ZCash address | Hidden from Avalanche |
| ZEC deposit amount | Only you and the protocol |
| USDC borrow amount | Public on Avalanche |
| ZCash-Avalanche link | Only the protocol knows |

**What it achieves**: This is the most important section for Morgan. It doesn't say "non-custodial" when it isn't. It explains the trade-off honestly. The bank escrow analogy is accessible. The privacy table is the detail Morgan needs to evaluate risk.

**Visual**: 2 key cards, privacy table, centered layout.

---

### 8. Team

**Sentiment**: personal trust. "There are real people behind this."

| | |
|---|---|
| Label | Team |
| Title | Built by Builders |

**2 members**:
- **Franco** — Smart contract architecture, ZK circuit design, protocol security
- **Seba** — ZCash integration, viewing key derivation, relayer service design

**What it achieves**: Putting a face (well, initials) to the builders. A small team focused on a specific niche communicates dedication. The focus areas show real technical expertise.

**Visual**: 2 cards with initial avatars, semi-transparent background.

---

### 9. CTA

**Sentiment**: open invitation. "Explore, verify, participate."

| | |
|---|---|
| Title | Explore ZLend |
| Subtitle | ZLend is in active development. Dive into the technical docs, verify the design, or join the conversation. |

**3 actions**: Technical Docs, GitHub, Join Community.

**What it achieves**: Not a "sign up now." It's "verify the code yourself" — exactly what Morgan wants. Acknowledges the project is in development (honesty). Offers multiple engagement paths.

---

## Changelog

| Date | Section | Change | Why |
|------|---------|--------|-----|
| 2026-02-22 | Problem | 3 generic cards → 3 scroll storytelling scenes | More visual impact, each scene with its own identity |
| 2026-02-22 | Problem | Title "DeFi Lending Has a Privacy Problem" → "Privacy Shouldn't Mean Exclusion" | Morgan's problem isn't losing privacy — it's being excluded from DeFi |
| 2026-02-22 | Problem | Title → "DeFi Is Growing. Where Does Your ZEC Fit?" | Tone should be discovery and curiosity, not negative |
| 2026-02-22 | Problem | Fear-based copy (MEV bots, whale watchers) → neutral observations | The sentiment is "OK I have capital, I could be earning, what do I do?" — never fear |
| 2026-02-22 | Problem | "No lending" → "No lending yet" | "Yet" implies possibility, not loss |
| 2026-02-22 | Problem | "No Good Options Today" scene removed | Generated negative, uncomfortable feelings — dead ends with red X don't fit a discovery tone |
| 2026-02-22 | Problem | Closing with "What if..." question instead of statement | Opens the door to the solution from curiosity, not frustration |
| 2026-02-22 | Market | Stats corrected: "290K ZEC wrapped" → "$0 DeFi access" and "100% ZEC stays on ZCash" | ZLend doesn't wrap or bridge ZEC — it uses ZK proofs. The original stats were misleading |

---

## Links

- User persona: [05_user-persona.md](05_user-persona.md)
- Problem doc: [02_problem.md](02_problem.md)
- Solution doc: [03_solution.md](03_solution.md)
- Market data: [08_market-data.md](08_market-data.md)
- Component source: `apps/landing/src/components/sections/`
- Content data: `apps/landing/src/data/content.ts`

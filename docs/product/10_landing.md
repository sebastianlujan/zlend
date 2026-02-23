# Landing Page

**Target persona**: Morgan — The ZCash Native ([05_user-persona.md](05_user-persona.md), Phase 1)
**Functional JTBD**: "I have ZEC, I want to use it in DeFi, I don't want to sacrifice my privacy"
**Overall sentiment**: discovery, curiosity, progressive trust. Never fear, never negativity.

The emotional journey is: "OK, I have capital → others are earning with theirs → my ZEC could be working in Avalanche DeFi → what can I do?" — all from curiosity, not from panic.

**Visual direction**: Cypherpunk aesthetic — scanline overlays, terminal-style UI elements, monospace typography for technical content, glowing accents, data-rain particle effects. Colors: red primary (#E84142), blue accent (#058AFF), dark surfaces. Three.js torus gateway + falling particle field.

---

## Sections

### 1. Hero

**Sentiment**: intrigue, immediate clarity.

| | |
|---|---|
| Headline | Access DeFi. Keep your ZEC. |
| Subheadline | Lock your ZCash. Get liquidity on Avalanche. Privacy preserved. |
| CTA | See How It Works → #solution |
| Secondary | Read the Docs → #cta |

**What it achieves**: Direct, grounded value proposition. Morgan understands instantly — DeFi access without giving up their ZEC. The typewriter animation on the headline creates a cypherpunk first impression. Scanline overlay + 3D torus gateway communicate tech without words.

**Visual**: Full-screen, scanline overlay, 3D scene background (Three.js torus + data-rain particles), typewriter headline animation with glow, gradient overlay. Dark.

---

### 2. Problem (The Landscape)

**Sentiment**: discovery, realization. "Oh right — my ZEC can't do any of this. But it could."

| | |
|---|---|
| Label | The Landscape |
| Title | Your ZEC Has Privacy. It Has Almost Nothing Else. |
| Subtitle | Yield, swaps, farming, governance — all happening on Avalanche. Your ZEC can't touch any of it. Until now. |

**Scene 1 — "What Your ZEC Gives You"**
Terminal-style header `// ZEC_INVENTORY` above a checklist card. Two sections: what ZEC gives you (✓ Privacy, Self-Custody, Censorship Resistance, Sound Money) and what it doesn't (✗ Yield, Swaps, Lending, Farming, Liquidity Pools, Governance, Payments, Cross-chain Access). Footer: "4 strengths. 8 blind spots."

**Scene 2 — "Lock ZEC. Get Stables. Access Everything."**
Side-by-side comparison: "ZEC Alone" (dashed border, dim — Privacy, Shielded Transactions, Self-Custody, Censorship Resistance) vs "ZEC + Stables on Avalanche" (solid primary border, bright — Yield, Swaps, Liquidity Pools, Farming, Governance, Payments, Cross-chain Bridges, Full Avalanche DeFi). Closes with: "Your ZEC stays locked. Your stables open every door on Avalanche."

**What it achieves**: Morgan sees the full picture — ZEC is strong on privacy but locked out of everything else. The contrast between 4 capabilities and 8 blind spots is immediate. The second scene shows what locking ZEC and getting stables unlocks: not just lending, but the entire Avalanche DeFi ecosystem. Leads naturally into Solution.

**Key design decisions**:
- Not framed around lending — framed around total DeFi access
- No fear-based language — observational, factual inventory
- Terminal `// ZEC_INVENTORY` header maintains cypherpunk aesthetic
- ✓/✗ checklist makes the gap visceral at a glance
- "ZEC Alone" card (dashed, dim) vs "ZEC + Stables" card (solid, bright) — visual weight mirrors the capability gap
- Closing line emphasizes: ZEC stays safe, stables do the work

---

### 3. How It Works (Solution)

**Sentiment**: clarity, simplicity. "Oh, it's just 3 steps."

| | |
|---|---|
| Label | How It Works |
| Title | Lock. Unlock. Access. |
| Subtitle | Your ZEC stays private. Your liquidity moves freely. |

**3 steps** (terminal-style cards with `> 01_` prefix):
1. **Lock** — Send ZEC to OGBank escrow. Secured by the protocol, verified by your viewing key.
2. **Unlock Liquidity** — A ZK proof is generated in your browser. USDC arrives on Avalanche — ready for any DeFi protocol.
3. **Return** — Done with DeFi? Return the USDC. Your private ZEC is released. Only when you choose.

**Key narrative decisions**:
- Not "borrowing" — it's unlocking liquidity to access DeFi products
- Not "repaying" — it's returning USDC to reclaim your private capital
- "Only when you choose" — the return is optional, not a debt obligation framing

**What it achieves**: Reduces complexity to 3 actions with the right mental model. Lock/unlock/return is intuitive. Morgan appreciates that it doesn't frame this as debt.

**Visual**: 3 terminal-style columns with `border-t-2` accent, monospace step numbers, dashed connector lines between steps.

---

### 4. Under the Hood

**Sentiment**: technical confidence. "I understand the mechanics."

| | |
|---|---|
| Label | Under the Hood |
| Title | Zero-Knowledge. Full Access. |

**6 features**:
1. ZEC → Avalanche DeFi
2. ZK-Verified Solvency (Ultrahonk)
3. Built on Aave V3
4. MEV-Proof Positions
5. Identity Unlinkability
6. Escrow Custody

**What it achieves**: Technical deep dive for Morgan. Each feature addresses a different concern: access, security, integration, privacy, custody.

**Visual**: Card grid (2-3 columns), semi-transparent background, glowing section heading.

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

**What it achieves**: Name-dropping technologies Morgan knows. If you know these words, you trust.

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
| $0 | DeFi access for shielded ZEC — OGBank changes this |
| 100% | ZEC stays on ZCash — no wrapping, no synthetic tokens |
| 10-12K | Weekly shielded transactions — active, engaged users |

**Source**: Blockworks Analytics (with link).

**Visual**: StatCard grid (4 columns), monospace for numbers, glowing section heading.

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

**Visual**: 2 key cards, privacy table, glowing section heading.

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

**Visual**: 2 cards with initial avatars, semi-transparent background.

---

### 9. CTA

**Sentiment**: open invitation. "Explore, verify, participate."

| | |
|---|---|
| Title | Explore OGBank |
| Subtitle | OGBank is in active development. Dive into the technical docs, verify the design, or join the conversation. |

**3 actions**: Technical Docs, GitHub, Join Community.

---

## Visual Techniques

Premium animation and interaction layer built on GSAP + ScrollTrigger (migrated from Anime.js).

| Technique | Where | Reference |
|-----------|-------|-----------|
| Character-level entrance | Hero h1 | statementof.com looping hero |
| Character-level hover | All section h2 headings | statementof.com stagger links |
| Scroll-driven parallax | Floating geometric elements | techyscouts.com floating elements |
| Magnetic hover | Primary buttons | statementof.com, techyscouts.com |
| 3D tilt hover | Cards | statementof.com |
| Mouse-tracking glow | Cards (radial gradient follows cursor) | statementof.com |
| Layered 3D gyroscope | Hero scene (dual torus + icosahedron + orbital particles) | techyscouts.com depth layering |
| Data-rain particles | Hero scene (200 falling particles) | Cypherpunk aesthetic |
| Custom cursor | Desktop only (ring + dot, mix-blend-mode: difference) | statementof.com |
| Scroll progress bar | Fixed 2px bar at top | Common premium pattern |
| Varied section reveals | fadeUp, scaleUp, slideLeft/Right per section | Diagonal rhythm |
| Button glow pulse | Primary buttons (pulsing box-shadow) | Premium CTA attention |
| Section dividers | Animated line with glowing red dot | Spacing + visual rhythm |
| Custom scrollbar | 6px, dark track, accent thumb on hover | Premium detail |

All animations respect `prefers-reduced-motion: reduce`. Custom cursor hidden on touch devices.

---

## Changelog

| Date | Section | Change | Why |
|------|---------|--------|-----|
| 2026-02-23 | Problem | Landscape reframe: inventory + stables unlock | Focus shift from lending to full DeFi access. ZEC inventory (4 strengths, 8 blind spots) replaces lending transparency card. "Lock ZEC → Get Stables → Access Everything" replaces ETH vs ZEC APY comparison. |
| 2026-02-23 | All | Premium animation upgrade (GSAP migration) | Char-level animations, scroll parallax, magnetic/tilt hovers, layered 3D, custom cursor, section dividers. |
| 2026-02-22 | All | Narrative reframe + cypherpunk aesthetic | Lock/unlock/return replaces borrow/repay. Cypherpunk visual identity. |
| 2026-02-22 | Hero | "Your ZEC. Unlocked." + typewriter animation | Short, punchy. Typewriter creates cypherpunk first impression. |
| 2026-02-22 | Solution → How It Works | Steps: Lock / Unlock Liquidity / Return | Not borrowing — unlocking liquidity. Not repaying — returning to reclaim private capital. |
| 2026-02-22 | How It Works → Under the Hood | "Zero-Knowledge. Full Access." | Renamed to avoid confusion with Solution section. |
| 2026-02-22 | Problem | "Your ZEC Has Privacy. It Lacks Everything Else." | Punchy. Highlights the gap without fear. |
| 2026-02-22 | Problem | Terminal `// FULLY VISIBLE` + blinking cursor | Cypherpunk treatment for the position card. |
| 2026-02-22 | Visual | Scanlines, glow text, data-rain particles, torus gateway | Cypherpunk aesthetic throughout. |
| 2026-02-22 | All | Bracket badges `[ LABEL ]`, section heading glow lines | Terminal-inspired UI elements. |

---

## Links

- User persona: [05_user-persona.md](05_user-persona.md)
- Problem doc: [02_problem.md](02_problem.md)
- Solution doc: [03_solution.md](03_solution.md)
- Market data: [08_market-data.md](08_market-data.md)
- Component source: `apps/landing/src/components/sections/`
- Content data: `apps/landing/src/data/content.ts`

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
Terminal-style `// ZEC_INVENTORY` card. What you have (✓ Privacy, Self-Custody, Sound Money) and opportunities waiting (→ Yield, Swaps, Farming, Liquidity Pools, Governance). Footer: "Strong foundations. Untapped opportunity."

**Scene 2 — "Lock ZEC. Get Stables. Access Everything."**
Side-by-side: "ZEC Alone" (dashed, dim — Privacy, Self-Custody, Sound Money) vs "ZEC + Stables on Avalanche" (primary, bright — Yield, Swaps, Farming, LPs, Governance). Closes with: "Your ZEC stays locked. Your stables open every door on Avalanche."

**What it achieves**: Morgan sees ZEC's strengths and the opportunity gap. Second scene shows what stables unlock. Leads into Solution.

**Key design decisions**:
- Framed as opportunity, not deficit — arrows (→) not crosses (✗)
- Terminal `// ZEC_INVENTORY` maintains cypherpunk aesthetic
- "ZEC Alone" (dashed, dim) vs "ZEC + Stables" (solid, bright) — visual weight mirrors capability gap

---

### 3. The Solution

**Sentiment**: clarity, simplicity. "Oh, it's just 3 steps."

| | |
|---|---|
| Label | The Solution |
| Title | Lock. Unlock. Access. |
| Subtitle | Your ZEC stays private. Your liquidity moves freely. |

**Visual flow diagram** showing the Zcash→OGBank→Avalanche path:

```
Desktop (horizontal):

  [Z]  ──▶  01 Lock  ──▶  [OGBank]  ──▶  02 Unlock  ──▶  [A]
  ZEC      (step desc)                   (step desc)      USDC
                              ◀──── 03 Return ────◀
                                   (step desc)

Mobile (vertical stack):

  [Z] ZEC
    │ 01 Lock
  [OGBank]
    │ 02 Unlock
  [A] USDC
    │ 03 Return
  ↑ Back to ZEC
```

**3 chain nodes** (`ChainNode.tsx`): Zcash (yellow `#F4B728` shield), OGBank (white lock), Avalanche (red `#E84142` triangle). Each in a 72px glow ring with hover effects.

**3 labeled arrows** (`FlowArrow.tsx`): SVG arrows with step number (`> 01_`), title, and description.

**3 steps**:
1. **Lock** — Send ZEC to OGBank escrow. Secured by the protocol, verified by your viewing key.
2. **Unlock Liquidity** — A ZK proof is generated in your browser. USDC arrives on Avalanche — ready for any DeFi protocol.
3. **Return** — Done with DeFi? Return the USDC. Your private ZEC is released. Only when you choose.

**Key narrative decisions**:
- Not "borrowing" — it's unlocking liquidity to access DeFi products
- Not "repaying" — it's returning USDC to reclaim your private capital
- "Only when you choose" — the return is optional, not a debt obligation framing

**What it achieves**: THE key product moment. Morgan sees a real diagram of capital flowing Zcash→OGBank→Avalanche and back. Chain logos make it tangible. Reduces complexity to 3 actions with the right mental model.

**Visual**: Flow diagram with inline SVG chain logos, `useScrollAnimation` fadeUp stagger. Desktop: horizontal with curved SVG return arc. Mobile: vertical stack.

---

### 4. Under the Hood

**Sentiment**: technical confidence. "I understand the mechanics."

| | |
|---|---|
| Label | Under the Hood |
| Title | Zero-Knowledge. Full Access. |
| Subtitle | From shielded ZEC to USDC on Aave — without exposing a single byte. |

**4-step pipeline diagram** (`PipelineNode.tsx` + `PipelineConnector.tsx`):

| Step | Label | Network | Description |
|------|-------|---------|-------------|
| 01 | Deposit ZEC | Zcash | Shielded escrow. Amount hidden. |
| 02 | Generate Proof | Browser | Ultrahonk ZK proof. Client-side. |
| 03 | Verify On-Chain | Avalanche | Smart contract verifies. Trustless. |
| 04 | Borrow USDC | Aave V3 | Proven collateral. Instant liquidity. |

**4 tech highlights** (`TechHighlights.tsx`): ZK-Verified Solvency, MEV-Proof Positions, Identity Unlinkability, Escrow Custody. Rendered as pill badges below the pipeline.

**What it achieves**: Technical deep dive for Morgan. The pipeline makes the ZK flow tangible — from shielded deposit to on-chain verification to liquidity. Highlights reinforce security properties.

**Visual**: Pipeline diagram with GSAP scroll-driven animations. SVG dashed connectors draw in on scroll, glowing particles travel along connector paths. Desktop: horizontal pipeline. Mobile: vertical stack. `useGSAP` + `ScrollTrigger` for timeline orchestration.

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
| Scroll-driven parallax | Floating geometric elements | techyscouts.com floating elements |
| Magnetic hover | Primary buttons | statementof.com, techyscouts.com |
| 3D tilt hover | Cards | statementof.com |
| Mouse-tracking glow | Cards (radial gradient follows cursor) | statementof.com |
| 3D shield model | Hero scene (Three.js shield + particle field) | Cypherpunk aesthetic |
| Custom cursor | Desktop only (ring + dot, mix-blend-mode: difference) | statementof.com |
| Scroll progress bar | Fixed 2px bar at top | Common premium pattern |
| Varied section reveals | fadeUp, fadeIn, scaleUp, slideLeft/Right per section | Diagonal rhythm |
| Button glow pulse | Primary buttons (pulsing box-shadow) | Premium CTA attention |
| Section dividers | Animated line with glowing red dot | Spacing + visual rhythm |
| Custom scrollbar | 6px, dark track, accent thumb on hover | Premium detail |
| Solution flow diagram | Solution section (chain logos in glow rings, SVG flow arrows) | Product clarity |
| Pipeline draw-in | Under the Hood (SVG connector stroke animation on scroll) | GSAP ScrollTrigger |
| Pipeline particle travel | Under the Hood (glowing dots travel along connector paths) | GSAP timeline |
| Count-up / typewriter stats | Market Data (numbers animate up, alternating typed values) | GSAP + IntersectionObserver |

All animations respect `prefers-reduced-motion: reduce`. Custom cursor hidden on touch devices.

---

## Changelog

| Date | Section | Change | Why |
|------|---------|--------|-----|
| 2026-02-23 | Solution | Visual flow diagram with chain logos | Replaced flat card grid with Zcash→OGBank→Avalanche diagram. Inline SVG chain logos (Zcash yellow, Avalanche red, OGBank white lock). sectionLabel "How It Works" → "The Solution". This is THE key product moment. |
| 2026-02-23 | Under the Hood | 4-step pipeline diagram replaces 6-feature grid | Pipeline: Deposit ZEC → Generate Proof → Verify On-Chain → Borrow USDC. GSAP scroll-driven connector draw-in + particle travel. 4 tech highlight badges. Short punchy descriptions. |
| 2026-02-23 | All | Removed character-level hover from headings | `useSplitTextHover` removed from all 8 section components. Cleaner, less distracting. |
| 2026-02-23 | All | Copy reduction — less text, more impact | Shortened descriptions across pipeline steps and features to 4-5 words. Removed verbose detail fields. |
| 2026-02-23 | Problem | Landscape reframe: inventory + stables unlock | Focus shift from lending to full DeFi access. ZEC inventory (strengths + opportunities) replaces lending transparency card. "Lock ZEC → Get Stables → Access Everything" replaces ETH vs ZEC APY comparison. |
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

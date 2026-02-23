# Landing Page

**Target persona**: Morgan — The ZCash Native ([05_user-persona.md](05_user-persona.md), Phase 1)
**Functional JTBD**: "I have ZEC, I want to use it in DeFi, I don't want to sacrifice my privacy"
**Overall sentiment**: discovery, curiosity, progressive trust. Never fear, never negativity.

The emotional journey is: "OK, I have capital → others are earning with theirs → my ZEC could be working in Avalanche DeFi → what can I do?" — all from curiosity, not from panic.

**Visual direction**: Cypherpunk terminal aesthetic — monospace typography (`JetBrains Mono`), terminal-style UI elements (`//` comments, `>` prompts, `_` suffixes), border-color-only hovers, dark surfaces with subtle radial glows. Colors: red primary (#E84142 / #FF394A), gold ZCash accent (#F4B728), blue accent (#058AFF), dark surfaces (#0a0a0c → #161617). Three.js 3D scene in Hero.

---

## Site Structure

3-page SPA with React Router v7. Global shell: Header (nav + "Read the Docs" CTA) + ScrollProgress bar + Footer.

| Page | Route | Sections | Purpose |
|------|-------|----------|---------|
| **Home** | `/` | Hero → Problem → Solution → HowItWorksCompact → CTA | Emotional hook: problem → solution → pipeline summary |
| **Technology** | `/technology` | HowItWorks → Features → CTA | Technical validation: architecture + primitives |
| **Market** | `/market` | MarketData → Trust → CTA | Opportunity + trust: market data + custody model |

Each page is separated by `SectionDivider` components (static red dot, dashed line).

---

## Home Page (`/`)

### 1. Hero

**Sentiment**: intrigue, immediate clarity.

| | |
|---|---|
| Headline | Access DeFi. Keep your ZEC. |
| Subheadline | Lock your ZCash. Get liquidity on Avalanche. Privacy preserved. |
| CTA | Go to App → # (placeholder until app is live) |
| Secondary | Read the Docs → /technology |

**What it achieves**: Direct, grounded value proposition. Morgan understands instantly — DeFi access without giving up their ZEC. The scanline overlay + 3D scene communicate tech without words.

**Visual**: Full-screen, scanline overlay, Three.js 3D scene background, GSAP fadeUp timeline (h1 → subtitle → CTAs staggered at -=0.2), gradient overlay. Dark.

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
Side-by-side: "ZEC Alone" (dashed border, dim — Privacy, Self-Custody, Sound Money) vs "ZEC + Stables on Avalanche" (primary border, bright — Yield, Swaps, Farming, LPs, Governance). Closes with: "Your ZEC stays locked. Your stables open every door on Avalanche."

**What it achieves**: Morgan sees ZEC's strengths and the opportunity gap. Second scene shows what stables unlock. Leads into Solution.

**Key design decisions**:
- Framed as opportunity, not deficit — arrows (→) not crosses (✗)
- Terminal `// ZEC_INVENTORY` maintains cypherpunk aesthetic
- "ZEC Alone" (dashed, dim) vs "ZEC + Stables" (solid, bright) — visual weight mirrors capability gap
- All hover effects are border-color only (`transition-colors duration-200`)

---

### 3. The Solution

**Sentiment**: clarity, simplicity. "Oh, it's just 3 steps."

| | |
|---|---|
| Label | The Solution |
| Title | Lock. Unlock. Access. |
| Subtitle | Your ZEC stays private. Your liquidity moves freely. |

**Split-world layout** showing the Zcash ↔ OGBank ↔ Avalanche bridge:

**Desktop (3-column grid):**
```
[Private World]  ···  [OGBank]  ···  [DeFi World]
// zcash_network      // bridge_protocol     // avalanche_network
     Zcash                 🔒                    Avalanche
```

**Tablet:** 2-column (worlds side by side) + bridge below.
**Mobile:** Vertical stack (Zcash → Bridge → Avalanche).

**2 world panels** (`WorldPanel.tsx`): Fixed height (`h-[200px] md:h-[240px]`), terminal tag at top (`// zcash_network` / `// avalanche_network`), large monospace title (`text-2xl md:text-3xl font-bold font-mono`), chain icon in `w-14 h-14` circle, radial gradient glow, dot-pattern overlay. Zcash uses gold (#F4B728), Avalanche uses primary red. Border hover `/20` → `/40`.

**1 bridge column** (`BridgeColumn.tsx`): Card wrapper with `// bridge_protocol` tag, lock icon (`w-16 h-16`), label `text-lg md:text-xl font-bold text-white font-mono`, dashed connector lines above/below (desktop only). Height matches WorldPanel for alignment.

**3 journey steps** (`JourneyStep.tsx`): LayerCard-style cards with dot indicator column (colored number + dot), network badge pill (Zcash/OGBank/Avalanche), title `text-xl md:text-2xl font-bold font-mono`, description, terminal prompt `> step_01_` that brightens on hover. Color-coded borders per side (gold/neutral/red).

**3 steps**:
1. **Lock** — Send ZEC to OGBank escrow. Secured by the protocol, verified by your viewing key. `[Zcash]`
2. **Unlock Liquidity** — A ZK proof is generated in your browser. USDC arrives on Avalanche — ready for any DeFi protocol. `[OGBank]`
3. **Return** — Done with DeFi? Return the USDC. Your private ZEC is released. Only when you choose. `[Avalanche]`

**Key narrative decisions**:
- Not "borrowing" — it's unlocking liquidity to access DeFi products
- Not "repaying" — it's returning USDC to reclaim your private capital
- "Only when you choose" — the return is optional, not a debt obligation framing

**What it achieves**: THE key product moment. Morgan sees two worlds (private ZCash, open DeFi) connected by OGBank. Reduces complexity to 3 actions. Network badges make each step's context clear.

---

### 4. Under the Hood — Compact (`HowItWorksCompact`)

**Sentiment**: technical teaser. "OK, I get the high-level flow."

Same header (label, title, subtitle) as the full version. Instead of expandable LayerCards, renders a **4-card grid** (`grid-cols-1 md:grid-cols-2 lg:grid-cols-4`) — each card shows step number, label, network badge, and one-line description. No click-to-expand, no DataFlowSpine, no TechHighlights. Color-coded borders (gold/accent/primary) match the full version.

Ends with a `> full_architecture →` link to `/technology` for users who want the deep dive.

**What it achieves**: Gives Morgan a quick mental model of the 4-step ZK pipeline without overwhelming the Home narrative. Curious users click through to Technology for the full expandable version.

---

## Technology Page (`/technology`)

### 5. Under the Hood — Full (`HowItWorks`)

**Sentiment**: technical confidence. "I understand the mechanics."

| | |
|---|---|
| Label | Under the Hood |
| Title | Zero-Knowledge. Full Access. |
| Subtitle | From shielded ZEC to USDC on Aave — without exposing a single byte. |

**4-layer card stack** (`LayerCard.tsx` + `LayerDetail.tsx` + `DataFlowSpine.tsx`):

| Step | Label | Network | Description |
|------|-------|---------|-------------|
| 01 | Deposit ZEC | Zcash | Shielded escrow. Amount hidden. |
| 02 | Generate Proof | Browser | Ultrahonk ZK proof. Client-side. |
| 03 | Verify On-Chain | Avalanche | Smart contract verifies. Trustless. |
| 04 | Borrow USDC | Aave V3 | Proven collateral. Instant liquidity. |

Each `LayerCard` is click-to-expand: shows terminal detail line (e.g. `zcash-cli z_sendmany ...`) and expanded description. Step indicator column with colored number + dot (gold for Zcash, accent for Browser/Aave, primary for Avalanche). Network badge pill.

`DataFlowSpine`: SVG vertical dashed line connecting the cards, stroke draws in on scroll (GSAP + ScrollTrigger, `power3.out`).

**4 tech highlights**: ZK-Verified Solvency, MEV-Proof Positions, Identity Unlinkability, Escrow Custody. Rendered as pill badges below the card stack.

**What it achieves**: Technical deep dive for Morgan. The card stack makes the ZK flow tangible — from shielded deposit to on-chain verification to liquidity. Click-to-expand lets curious users see terminal commands without cluttering the default view.

---

### 6. Features (Technical Foundation)

**Sentiment**: credibility. "They're using proven technology."

| | |
|---|---|
| Label | Technical Foundation |
| Title | Built on Proven Primitives |

**4 feature boxes** (`FeatureBox` in `Features.tsx`) — 2-column grid:

1. **Noir + Ultrahonk** `[ZK Proofs]` — ZK proofs generated client-side in Noir, verified on-chain by UltraHonk
2. **Aave V3 Integration** `[Infrastructure]` — No forked lending pool. Plugs directly into Aave V3 on Avalanche
3. **ZIP-32 Key Derivation** `[Cryptography]` — Deterministic address derivation from ZCash's hierarchical wallet standard
4. **Nullifier Protection** `[Security]` — Each lock cycle creates unique nullifier. Prevents replay attacks

**Box design**: Fixed height (`h-[240px]`), large monospace title (`text-2xl md:text-3xl font-bold font-mono`), tag at top in primary color. Resting state shows tag + title + `> details_` hint. On hover (desktop) or tap (mobile), an opacity overlay fades in revealing the full description. Accessible: `role="button"`, `tabIndex={0}`, `aria-expanded`, keyboard support (Enter/Space).

**What it achieves**: Name-dropping technologies Morgan knows. If you know these words, you trust. The hover-reveal pattern rewards curiosity without cluttering the initial view.

---

## Market Page (`/market`)

### 7. Market Data

**Sentiment**: validation. "There's real opportunity here."

| | |
|---|---|
| Label | Market Opportunity |
| Title | Untapped Capital, Waiting for Access |

**4 stats** (`StatCard` grid — 2 columns mobile, 4 columns desktop):

| Stat | Context |
|------|---------|
| 5.1M ZEC | In shielded pools (~$255M, up 5x in 2 years) |
| $0 | DeFi access for shielded ZEC — OGBank changes this |
| 100% | ZEC stays on ZCash — no wrapping, no synthetic tokens |
| 10-12K | Weekly shielded transactions — active, engaged users |

**Source**: Blockworks Analytics (with external link). Data accessed February 2026.

---

### 8. Trust

**Sentiment**: transparency, honesty. "They're honest about the trade-offs."

| | |
|---|---|
| Label | Trust Model |
| Title | Transparent About Trade-Offs |

**Custody model** (2 key cards):
- **Viewing Key** (you hold this) — verify balance, cannot move funds
- **Spending Key** (protocol holds this) — move ZEC in/out of escrow

**Privacy table**:
| Data | Who can see it |
|------|----------------|
| Your ZCash address | Hidden from Avalanche |
| ZEC deposit amount | Only you and the protocol |
| USDC borrow amount | Public on Avalanche |
| ZCash-Avalanche link | Only the protocol knows |

---

### 9. CTA (shared across all pages)

**Sentiment**: open invitation. "Explore, verify, participate."

| | |
|---|---|
| Title | Explore OGBank |
| Subtitle | OGBank is in active development. Dive into the technical docs, verify the design, or join the conversation. |

**3 actions**: Technical Docs, GitHub, Join Community.

---

## Animation System — "Terminal Render"

Unified across all 3 pages. One entrance, one hover, one easing.

| What | Implementation | Notes |
|------|----------------|-------|
| **Scroll entrance** | `fadeUp` — opacity 0→1, y 20→0, 0.5s, `power3.out` | Via `useScrollAnimation` hook. Trigger at `top 80%`. Children stagger at 0.1s. |
| **Hover** | Border-color shift only | `transition-colors duration-200`. Border goes from `/20` → `/40` opacity. No transforms, no shadows, no glow. |
| **Easing** | `power3.out` everywhere | Single easing for all GSAP animations. |
| **Reduced motion** | All animations respect `prefers-reduced-motion: reduce` | Hook `useReducedMotion()` skips GSAP setup entirely. |
| **3D scene** | Three.js in Hero only | Scanline overlay. Contained to Hero section. |
| **Scroll progress** | Fixed 2px bar at top | Tracks page scroll position. |
| **Section dividers** | Static red dot + dashed line | Between sections. No animation. |
| **DataFlowSpine** | SVG dashed line draw-in on scroll | HowItWorks section only. `power3.out`, trigger once. |
| **Custom scrollbar** | 6px, dark track, primary thumb on hover | CSS-only via `::-webkit-scrollbar`. |

**What was removed** (animation unification, 2026-02-23):
Character-level entrance, scroll-driven parallax, magnetic hover, 3D tilt hover, mouse-tracking glow, custom cursor, button glow pulse, varied section reveals (scaleUp/slideLeft/slideRight), pipeline particle travel, count-up/typewriter stats. 8 files deleted, CSS reduced by ~5KB.

---

## Changelog

| Date | Section | Change | Why |
|------|---------|--------|-----|
| 2026-02-23 | Under the Hood | Compact variant for Home | HowItWorksCompact: 4-card grid, no expandables, no spine. Full version stays on Technology. Link to `/technology` for deep dive. |
| 2026-02-23 | Solution | Terminal aesthetic redesign | WorldPanel: large mono titles, terminal tags, radial glow. BridgeColumn: card with connectors. JourneyStep: LayerCard structure with dot indicator, network badges. |
| 2026-02-23 | All | Animation unification ("Terminal Render") | ONE fadeUp entrance, ONE border-color hover, ONE power3.out easing. Removed char-split, parallax, magnetic, tilt, glow, cursor, button pulse, particles. 8 files deleted, CSS -5KB. |
| 2026-02-23 | Features | Hover-reveal box redesign | Fixed-height boxes (h-[240px]), opacity overlay, `> details_` affordance, mobile tap support, full accessibility. |
| 2026-02-23 | All | 3-page SPA restructure | Home (Hero→Problem→Solution→CTA), Technology (HowItWorks→Features→CTA), Market (MarketData→Trust→CTA). Header nav + footer shared. |
| 2026-02-23 | Under the Hood | LayerCard expandable redesign | Pipeline nodes → expandable LayerCards with terminal details. DataFlowSpine draw-in (no particles). |
| 2026-02-23 | Team | Section removed | Team section and data removed from the site. |
| 2026-02-23 | Solution | Visual flow diagram with chain logos | Replaced flat card grid with Zcash→OGBank→Avalanche split-world diagram. |
| 2026-02-23 | Under the Hood | 4-step pipeline diagram replaces 6-feature grid | Pipeline: Deposit ZEC → Generate Proof → Verify On-Chain → Borrow USDC. |
| 2026-02-23 | All | Copy reduction — less text, more impact | Shortened descriptions across pipeline steps and features to 4-5 words. |
| 2026-02-23 | Problem | Landscape reframe: inventory + stables unlock | ZEC inventory (strengths + opportunities) + "Lock ZEC → Get Stables → Access Everything". |
| 2026-02-22 | All | Narrative reframe + cypherpunk aesthetic | Lock/unlock/return replaces borrow/repay. Cypherpunk visual identity. |

---

## Links

- User persona: [05_user-persona.md](05_user-persona.md)
- Problem doc: [02_problem.md](02_problem.md)
- Solution doc: [03_solution.md](03_solution.md)
- Market data: [08_market-data.md](08_market-data.md)
- User journey: [06_user-journey.md](06_user-journey.md)
- Component source: `apps/landing/src/components/sections/`
- Content data: `apps/landing/src/data/content.ts`

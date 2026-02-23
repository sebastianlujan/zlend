# Plan: Update product docs to match current site

## Context

The landing site has evolved significantly (3-page SPA, animation unification, Solution/Features/HowItWorks redesign, Team section removed) but the product docs haven't been updated to match. Two docs need updating:

1. **`docs/product/10_landing.md`** — heavily outdated (wrong page structure, deleted components, removed animations)
2. **`docs/product/06_user-journey.md`** — doesn't reference the landing site's role in the journey, can be improved

---

## Part 1: Update `10_landing.md`

### What's outdated

| Area | Doc says | Actual site |
|------|----------|-------------|
| **Structure** | 1 page, 9 sections (Hero→Problem→Solution→Under the Hood→Features→Market→Trust→Team→CTA) | 3 pages: Home (`/`: Hero→Problem→Solution→CTA), Technology (`/technology`: HowItWorks→Features→CTA), Market (`/market`: MarketData→Trust→CTA) |
| **Team section** | Section 8 with Franco & Seba | **Removed** — no component, no data |
| **Hero animation** | "typewriter animation on the headline" | Simple fadeUp timeline (typewriter removed) |
| **Hero secondary CTA** | "Read the Docs → #cta" | "Read the Docs → /technology" |
| **Solution components** | ChainNode.tsx + FlowArrow.tsx with 72px glow rings, SVG arrows | WorldPanel.tsx (large mono title, terminal tag `// zcash_network`, radial glow, border hover), BridgeColumn.tsx (card with `// bridge_protocol`, lock icon, dashed connectors), JourneyStep.tsx (dot indicator, network badge pill, `> step_01_` terminal hint) |
| **Under the Hood** | PipelineNode.tsx + PipelineConnector.tsx + particle travel | LayerCard.tsx (expandable click-to-reveal), LayerDetail.tsx (terminal line), DataFlowSpine.tsx (draw-in only, no particles) |
| **Features design** | "2 columns, monospace tags, hover effects" | Fixed-height boxes (h-[240px]), large mono titles, opacity overlay reveal on hover/tap, `> details_` affordance, accessible (`role="button"`, `aria-expanded`, keyboard support) |
| **Visual Techniques** | 15 techniques including char-split, parallax, magnetic hover, 3D tilt, glow tracking, custom cursor, button glow pulse, pipeline particles, varied reveals | Unified "Terminal Render": ONE fadeUp scroll entrance, ONE border-color hover, ONE power3.out easing. 8 files deleted, CSS shrunk ~5KB |

### Changes to make

1. **Restructure sections to reflect 3-page architecture** — Group under Home, Technology, Market headings
2. **Remove Team section** entirely (section 8)
3. **Update Hero** — Remove typewriter reference, fix secondary CTA target, describe fadeUp timeline
4. **Rewrite Solution** — Replace ChainNode/FlowArrow with WorldPanel/BridgeColumn/JourneyStep descriptions. Document terminal aesthetic, responsive layouts (desktop 3-col grid, tablet 2+1, mobile vertical stack)
5. **Rewrite Under the Hood / HowItWorks** — Replace PipelineNode/PipelineConnector with LayerCard/LayerDetail/DataFlowSpine. Describe expandable terminal details, vertical card stack
6. **Rewrite Features** — Document fixed-height overlay boxes, accessibility, hover UX pattern
7. **Replace Visual Techniques table** — Document unified animation system. List what was removed and what remains
8. **Add changelog entries** for animation unification (2026-02-23) and Solution redesign (2026-02-23)
9. **Update Visual direction** line at top — remove references to "data-rain particle effects", "Three.js torus gateway"

---

## Part 2: Improve `06_user-journey.md`

### What can be improved

The user journey doc covers the full product lifecycle (Awareness → Due Diligence → Onboarding → First Borrow → Active Use → Claim → Liquidation) but doesn't map the **landing site's role** in the early stages.

### Changes to make

1. **Stage 1 (Awareness) — Add landing site touchpoint**: After Morgan discovers OGBank via ZCash channels, they visit the landing. The Home page (Hero → Problem → Solution) tells the story: "I have capital → my ZEC could be working → here's how". Reference the 3-page structure.

2. **Stage 2 (Due Diligence) — Map to landing pages**: Morgan's 30-60 minute review now has specific landing pages:
   - Technology page → HowItWorks (4-step pipeline) + Features (proven primitives)
   - Market page → MarketData (opportunity validation) + Trust (custody model, privacy table)
   - This supplements the GitHub/code review

3. **Stage 2 "What They See" — Update references**: Currently lists generic "technical documentation, open-source code, trust model diagram". Update to reference the actual sections on the landing site that serve this purpose.

4. **Navigation flow in Awareness→Due Diligence transition**: Morgan arrives at Home (emotional hook), clicks "See How It Works" to Solution section or "Read the Docs" to Technology page. Technology page provides technical validation. Market page provides trust validation. Each page ends with CTA to explore further.

5. **Minor refresh**: Update any references to "lending" framing that should be "lock/unlock" framing (checking for consistency with the reframe done in the landing site).

---

## Files to modify

| File | Scope |
|------|-------|
| `docs/product/10_landing.md` | Near-full rewrite: 3-page structure, updated components, updated animations, remove Team, add changelog |
| `docs/product/06_user-journey.md` | Add landing site mapping to Stages 1-2, update "What They See" sections |

## Verification

1. Read the updated docs end-to-end to ensure they match what the site actually shows
2. Cross-check component names and file paths against `apps/landing/src/components/sections/`
3. Ensure changelog dates are correct (2026-02-23 for recent changes)
4. Verify links to other product docs still resolve correctly

# User Personas

## Growth Strategy: Why Persona Order Matters

Two personas. Two phases. The order is not arbitrary — it's driven by data.

### The Data

| Signal | What It Tells Us |
|--------|-----------------|
| **290K ZEC** already wrapped on other chains (Solana, BSC, Near) | ZCash holders are *already* sacrificing privacy for DeFi access. Proven demand. |
| **0 ZEC** on Avalanche today | Greenfield. No competition for these users on our target chain. |
| **5.1M ZEC** (~$255M) in shielded pools | Large addressable pool of users who hold the asset we need — and capital that could flow to Avalanche. |
| **400-1,000 daily** shielded transactions | Active engagement, not dormant holders. |

### The Logic

| Factor | Morgan (ZCash Native) | Alex (Private Holder) |
|--------|----------------------|----------------------|
| Already has ZEC | Yes (in shielded pool) | Maybe (would need to buy) |
| Understands the problem | Lives it daily | Needs to be educated |
| Steps to first borrow | 2 (see OGBank exists → verify code) | 5+ (learn ZEC → buy → wallet → DeFi → trust) |
| Where to find them | ZCash Discord, forums, Ywallet/Zingo | Crypto Twitter, SEO, paid ads |
| Cost of acquisition | Low (concentrated community) | High (diffuse market) |
| Average deposit size | High (significant ZEC positions) | Low (testing with small amounts) |
| Validates our tech | Yes (reads circuits, gives feedback) | No (trusts marketing or doesn't) |
| Risk of churn | Low (no alternative exists) | High (any friction → they leave) |

**Conclusion:** Morgan validates product-market fit. Alex scales it. Building for Alex first means spending engineering effort on UX polish for a user who doesn't exist yet, while ignoring the user who's already waiting.

### Phased Approach

```
Phase 1: LAUNCH (Morgan)          Phase 2: GROWTH (Alex)
─────────────────────             ──────────────────────
Goal: Product-market fit          Goal: Market expansion
Users: ZCash community            Users: Crypto-curious holders
Channel: Forums, Discord,         Channel: SEO, content,
  developer announcements            partnerships, ads
Message: "Your ZEC now works      Message: "Put your crypto to
  in Avalanche DeFi"                work on Avalanche."
Landing: Technical credibility    Landing: Simplicity + trust
  + honest trade-offs               + social proof from Phase 1
Success metric: First 100         Success metric: 1,000+ users,
  deposits from ZCash natives       organic traffic, retention
```

**Phase 1 unlocks Phase 2.** "Used by the ZCash community" and "100+ deposits from ZCash-native users" are the trust signals Alex needs to overcome anxiety. You can't manufacture that trust — you earn it from Morgan first.

---

## Launch Persona: "The ZCash Native"

> Morgan is who we build for at launch. They validate the protocol, stress-test the cryptography, and become the trust signal that unlocks the broader market.

### Profile

| Attribute | Description |
|-----------|-------------|
| **Name** | Morgan |
| **Age** | 25-45 |
| **Crypto experience** | Deeply crypto-native. Has used DeFi on Ethereum. Runs their own ZCash node or uses Ywallet/Zingo. Understands shielded transactions, viewing keys, and the privacy model. |
| **Privacy stance** | Privacy is a core belief, not just a preference. Follows ZCash governance. Reads ZIP proposals. May contribute to privacy-focused projects. |
| **Financial behavior** | Most holdings in ZEC (shielded pool). Some ETH for DeFi. Frustrated that ZEC can't participate in DeFi — and wants to put that capital to work. |
| **Technical skill** | High. Comfortable with command-line tools, key management, and reading smart contract code. |

### Situation & Trigger

Morgan has significant ZEC in shielded pools. They see Aave lending rates and wish they could earn yield or borrow against their ZEC. They've been waiting for a way to use their ZEC in DeFi. They follow ZCash community channels and hear about OGBank through a forum post or developer announcement.

Some Morgans have already wrapped ZEC on Solana or BSC — breaking their privacy — just to access basic DeFi. They did it reluctantly. OGBank gives them a path that doesn't require that sacrifice — and routes their capital through Avalanche.

### Jobs To Be Done

| Job Type | Job Statement |
|----------|--------------|
| **Functional** | "When I have ZEC that I can't use in DeFi, I want to borrow against it on Avalanche while keeping my privacy, so I can finally put my capital to work." |
| **Emotional** | "When a new protocol asks me to deposit my ZEC, I want to verify the escrow and proof system myself, so I can trust it based on math and code, not marketing." |
| **Social** | "When I recommend a protocol to the ZCash community, I want it to be technically sound and honest about its trade-offs, so I maintain credibility." |

### Forces of Progress

```
DRIVING ADOPTION                    RESISTING ADOPTION
─────────────────                   ──────────────────

PUSH                                ANXIETY
• ZEC locked out of all DeFi        • "The protocol holds my spending
• Watching ETH holders earn            key — that's custodial risk"
  yield while ZEC sits idle          • "Is Ultrahonk actually secure?"
• Privacy breaks when bridging      • "What if the relayer gets
  to any other chain                   compromised?"
• Already wrapped ZEC on other
  chains (privacy sacrifice)

PULL                                HABIT
• "Finally, my ZEC works in         • "My ZEC is safe in shielded
  Avalanche DeFi"                      pool. Why take any risk?"
• "Ultrahonk is Aztec's tech —      • "I've survived without DeFi
  I trust the cryptography"            this long"
• "I can verify the Noir             • "Every new protocol is a
  circuits myself"                      potential honeypot"
• "On-chain proof of obligation
  creates accountability"
```

### What Morgan Needs From Us

1. **Technical transparency** — Open-source Noir circuits. Verifiable on-chain proofs. Published security model.
2. **Honest trust model** — Don't say "non-custodial" when it's not. Clearly state: protocol holds spending key, here's why, here are the mitigations, here's the roadmap to multi-sig.
3. **ZCash-native integration** — Works with existing ZCash wallets. Supports shielded addresses natively.
4. **Community alignment** — Engagement with ZCash forums. Respect for the privacy ethos. Honest about trade-offs.

### How We Reach Morgan

| Channel | Action |
|---------|--------|
| ZCash Community Forum | Technical post explaining the protocol. Link to circuits. Honest about trade-offs. |
| ZCash Discord | Direct engagement. Answer technical questions. |
| Ywallet / Zingo communities | Integration discussions. Viewing key compatibility. |
| Developer conferences (Zcon) | Demos. Code walkthroughs. |
| GitHub | Open-source everything. Invite code review. |

---

## Growth Persona: "The Private Holder"

> Alex is who we scale to after launch. They arrive when Morgan has already validated the protocol and social proof exists. They don't need to understand ZK proofs — they need to trust the app.

### Profile

| Attribute | Description |
|-----------|-------------|
| **Name** | Alex |
| **Age** | 28-40 |
| **Crypto experience** | Crypto-curious. Holds crypto on exchanges (Binance, Coinbase). Has used MetaMask once or twice. Heard of DeFi but hasn't used it actively. |
| **Privacy stance** | Values privacy as a principle. Uses ad blockers, avoids sharing financial info, uncomfortable with how much data apps collect. Not a "privacy maximalist" — just doesn't want their finances to be public. |
| **Financial behavior** | Holds a mix of BTC, ETH, and some ZEC. Doesn't day-trade. Buys and holds. Sometimes needs liquidity but doesn't want to sell. |
| **Technical skill** | Can use a web app. Understands the concept of a wallet and an address. Does NOT understand ZK proofs, shielded transactions, or DeFi mechanics. |

### Situation & Trigger

Alex has $15,000 in ZCash sitting in a wallet. They need $5,000 for a down payment on something but don't want to sell their ZEC (they think it will go up). They Google "borrow against crypto" and find that all options either:
- Require full transparency (Aave) — they're uncomfortable with that
- Require handing over custody (Nexo, Ledn) — they remember FTX
- Don't accept ZCash at all

They're stuck. They either sell (and lose the position) or don't borrow (and miss the opportunity).

### Jobs To Be Done

| Job Type | Job Statement |
|----------|--------------|
| **Functional** | "When I need cash but don't want to sell my crypto, I want to borrow against it without revealing my holdings, so I can access liquidity while keeping my position." |
| **Emotional** | "When I interact with financial apps, I want to feel like my financial life is private, so I don't feel exposed or surveilled." |
| **Social** | "When I talk to friends about crypto, I want to be able to say I use a protocol that respects privacy, so I feel aligned with my values." |

### Forces of Progress

```
DRIVING ADOPTION                    RESISTING ADOPTION
─────────────────                   ──────────────────

PUSH (pain with current)            ANXIETY (fear of new)
• ZEC sitting idle with no yield    • "OGBank holds my ZEC during the
• Uncomfortable with Aave's           loan — what if they don't
  transparency                        return it?"
• Don't trust CeFi after            • "I don't understand the
  FTX/Celsius                         technology behind it"
                                    • "Will regulators shut this down?"

PULL (attraction to new)            HABIT (comfort with old)
• "My ZEC finally works for me"     • "My ZEC is fine just sitting
• "Nobody sees my position"           in my wallet"
• "It looks like a normal app"      • "I've never used DeFi before"
• "Built on Aave V3 — sounds        • "Exchanges are easy enough"
  legit"
• "Used by the ZCash community"
  ← THIS IS THE PHASE 1 UNLOCK
```

### What Alex Needs From Us

1. **Simple UX** — No jargon. No ZK terminology. "Deposit, Borrow, Repay" — that's it.
2. **Trust signals** — "Built on Aave V3" (they've heard of it). Audit reports. On-chain proof of obligation. "Trusted by ZCash community" (earned from Phase 1).
3. **Education** — Brief explainer on how the escrow works. Not a whitepaper. A 30-second video or 3-step infographic.
4. **Safety net** — Clear information about liquidation. Alerts before it happens. No surprises.
5. **Custodial transparency** — Honest explanation that OGBank holds ZEC during the loan, why, and what protections exist.

### How We Reach Alex

| Channel | Action | Prerequisite |
|---------|--------|-------------|
| SEO / content marketing | "How to borrow against ZCash", "Use ZEC in DeFi" | Protocol live + social proof |
| Crypto Twitter / X | User stories. Simple explainers. Not technical threads. | Morgan testimonials to reference |
| YouTube / TikTok | 60-second "how it works" demo | Polished UX (Phase 2 investment) |
| Partnerships | Wallet integrations (MetaMask Snaps, etc.) | Kohaku adapter + proven demand |
| Referral program | Morgan refers Alex (community → mainstream) | Active user base from Phase 1 |

---

## Anti-Persona: Who Is NOT Our User

| Anti-Persona | Why Not |
|-------------|---------|
| **Day traders** | Need speed, not privacy. Transparency helps them (they want to see liquidation levels to trade around them). |
| **MEV operators** | They profit from the transparency OGBank removes. We're actively working against their interests. |
| **Institutional desks requiring full audit trails** | They need provable, auditable transparency for compliance. Our protocol doesn't provide that level of reporting. |
| **Users who don't own crypto** | OGBank requires ZCash as collateral. We're not an onramp. |
| **Users seeking anonymity for illicit purposes** | OGBank is private, not anonymous. The protocol knows the escrow relationship. On-chain proofs create auditable records. |

---

## Design Rules

Now that persona priority is phased, design decisions follow accordingly:

**Phase 1 (Morgan-first):**
- Prioritize technical correctness over UX polish
- Document everything — trust model, circuits, trade-offs
- The app can be functional but spartan. Morgan doesn't need animations.
- Invest in: security, open-source, community engagement

**Phase 2 (Alex-expansion):**
- Invest in UX simplification — hide complexity behind "Deposit, Borrow, Repay"
- Add onboarding flows, tooltips, progress indicators
- Create educational content (video, infographics)
- Polish the landing page for non-technical audiences

**Cross-phase rule:** If a feature helps Morgan but confuses Alex, ship it now with technical docs. If a feature helps Alex but Morgan hasn't validated the underlying protocol yet, defer it until Phase 2.

---

## Links

- Problem they face: [02_problem.md](02_problem.md)
- How we solve it: [03_solution.md](03_solution.md)
- Their journey: [06_user-journey.md](06_user-journey.md)
- Market data supporting this strategy: [08_market-data.md](08_market-data.md)

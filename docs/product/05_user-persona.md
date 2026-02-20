# User Personas

## Primary Persona: "The Private Holder"

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
• ZEC sitting idle with no yield    • "Is the ZK math actually safe?"
• Uncomfortable with Aave's         • "What if I lose my collateral?"
  transparency                      • "Will regulators shut this down?"
• Don't trust CeFi after            • "I don't understand the
  FTX/Celsius                         technology behind it"

PULL (attraction to new)            HABIT (comfort with old)
• "My ZEC finally works for me"     • "My ZEC is fine just sitting
• "Nobody sees my position"           in my wallet"
• "It looks like a normal app"      • "I've never used DeFi before"
• "My keys stay with me"            • "Exchanges are easy enough"
```

### What Alex Needs From Us

1. **Simple UX** — No jargon. No ZK terminology. "Lock, Borrow, Repay" — that's it.
2. **Trust signals** — "Built on Aave V3" (they've heard of it). "Your keys never leave your browser." Audit reports.
3. **Education** — Brief explainer on how privacy works. Not a whitepaper. A 30-second video or 3-step infographic.
4. **Safety net** — Clear information about liquidation. Alerts before it happens. No surprises.

---

## Secondary Persona: "The ZCash Native"

### Profile

| Attribute | Description |
|-----------|-------------|
| **Name** | Morgan |
| **Age** | 25-45 |
| **Crypto experience** | Deeply crypto-native. Has used DeFi on Ethereum. Runs their own ZCash node or uses Ywallet/Zingo. Understands shielded transactions, viewing keys, and the privacy model. |
| **Privacy stance** | Privacy is a core belief, not just a preference. Follows ZCash governance. Reads ZIP proposals. May contribute to privacy-focused projects. |
| **Financial behavior** | Most holdings in ZEC (shielded pool). Some ETH for DeFi. Frustrated that ZEC can't participate in DeFi without breaking privacy. |
| **Technical skill** | High. Comfortable with command-line tools, key management, and reading smart contract code. |

### Situation & Trigger

Morgan has significant ZEC in shielded pools. They see Aave lending rates and wish they could earn yield or borrow. They've been waiting for something like ZLend. They follow ZCash community channels and hear about ZLend through a forum post or developer announcement.

### Jobs To Be Done

| Job Type | Job Statement |
|----------|--------------|
| **Functional** | "When I have shielded ZEC that I can't use in DeFi, I want to borrow against it while maintaining my shielded status, so I can access liquidity without compromising my privacy model." |
| **Emotional** | "When a new privacy protocol launches, I want to verify the cryptographic design myself, so I can trust it based on math, not marketing." |
| **Social** | "When I recommend a protocol to the ZCash community, I want it to align with our privacy values, so I maintain credibility." |

### Forces of Progress

```
DRIVING ADOPTION                    RESISTING ADOPTION
─────────────────                   ──────────────────

PUSH                                ANXIETY
• ZEC locked out of all DeFi        • "Is Ultrahonk actually secure?"
• Watching ETH holders earn          • "Does the relayer compromise
  yield while ZEC sits idle            the threat model?"
• Privacy breaks when bridging      • "Avalanche isn't ZCash — is
  to any other chain                   the trust assumption acceptable?"

PULL                                HABIT
• "Finally, private lending"         • "My ZEC is safe in shielded
• "Ultrahonk is Aztec's tech —         pool. Why take any risk?"
  I trust the cryptography"          • "I've survived without DeFi
• "I can verify the Noir               this long"
  circuits myself"                   • "Every new protocol is a
• "Privacy Pools for compliance"        potential honeypot"
```

### What Morgan Needs From Us

1. **Technical transparency** — Open-source Noir circuits. Verifiable on-chain proofs. Published security model.
2. **ZCash-native integration** — Works with existing ZCash wallets. Supports shielded addresses natively.
3. **Honest threat model** — What the relayer can see. What the contract reveals. Where the trust assumptions are.
4. **Community alignment** — Engagement with ZCash forums. Respect for the privacy ethos.

---

## Anti-Persona: Who Is NOT Our User

| Anti-Persona | Why Not |
|-------------|---------|
| **Day traders** | Need speed, not privacy. Transparency helps them (they want to see liquidation levels to trade around them). |
| **MEV operators** | They profit from the transparency ZLend removes. We're actively working against their interests. |
| **Institutional desks requiring full audit trails** | They need provable, auditable transparency for compliance. Our selective disclosure model may not satisfy their requirements (yet). |
| **Users who don't own crypto** | ZLend requires ZCash as collateral. We're not an onramp. |
| **Users seeking anonymity for illicit purposes** | ZLend includes Privacy Pools and compliance hooks. We're building privacy, not anonymity. The protocol supports selective disclosure. |

---

## Persona Priority

For **MVP**, we design primarily for **Alex (The Private Holder)** — crypto-curious, values simplicity, needs privacy but not deep technical control.

**Morgan (The ZCash Native)** will adopt naturally if the cryptography is sound. They don't need UX hand-holding — they need verifiable security.

**Design rule:** If a feature choice helps Alex but confuses Morgan, we ship it. If it helps Morgan but confuses Alex, we add it as an advanced option or defer it.

---

## Links

- Problem they face: [02_problem.md](02_problem.md)
- How we solve it: [03_solution.md](03_solution.md)
- Their journey: [06_user-journey.md](06_user-journey.md)

# User Journey

## TL;DR

**Persona:** Morgan — ZCash native, DeFi-literate, holds shielded ZEC, wants yield without breaking privacy.

**The cycle:** Discover OGBank → verify the code → deposit ZEC → borrow USDC on Avalanche → manage the position → repay → get ZEC back.

**9 stages, 3 critical moments:**

| Stage | One-liner |
|-------|-----------|
| 1. Discovery | Finds OGBank on ZCash Forum/Discord. Stays if technical, leaves if marketing. |
| 2. Due diligence | 30-60 min reviewing code, trust model, liquidation mechanics. Decision point. |
| 3. Onboarding | Deposits small test amount. Friction = verifying escrow address, not wallet UX. |
| 4. First borrow | Borrows USDC. Inspects ZK proof. 10-30s proof generation wait. |
| 5. Active management | Monitors ratio, rates, oracle. Reacts to market: ZEC up (borrow more?), ZEC down (repay?), rate spike (close?). |
| **6. Undercollateralization** | **ZEC drops. Decides: partial repay, add collateral, wait, or accept liquidation. Cross-chain latency (~75s) is the enemy.** |
| 7. Return & claim | Repays USDC + interest. Verifies ZEC return independently via viewing key. |
| **8. Liquidation** | **Verifies fairness post-hoc: correct price? honest oracle? right penalty? One bad liquidation kills Phase 1 adoption.** |
| 9. Re-engagement | If fair experience → deposits more. If broken trust → never returns + tells community. |

**Key risks:** Oracle reliability (existential), relayer uptime (claim depends on it), cross-chain latency during crashes, Aave pause freezing positions.

**Open design questions that affect UX:** Who triggers liquidation? Partial vs full? How is seized ZEC converted to USDC? Can users add collateral to existing positions? Incremental borrows?

---

This journey follows **Morgan (The ZCash Native)** — Phase 1 launch persona. Technically proficient ZCash holder, DeFi-literate, wants yield without breaking privacy.

Alex (Phase 2, crypto-curious) is included at the end as a future reference. NOT the MVP design target.

---

## Stage 1: Discovery

> "Finally — someone is building DeFi access for ZCash without breaking privacy."

### Trigger
Morgan sees a technical post on ZCash Forum, a mention in ZCash Discord, or a Zcon talk about "locking shielded ZEC to unlock liquidity on Avalanche." They've probably already wrapped ZEC on Solana or BSC reluctantly and want something better.

### Channels
- ZCash Community Forum (primary)
- ZCash Discord
- Zcon / privacy conferences
- GitHub (open-source code drops)
- Word of mouth from other ZCash developers

### What they think
- "How does the escrow work? Who holds the spending key?"
- "Which ZK proving system? Is it audited?"
- "Can I verify the circuits myself?"
- "Is this another custodial honeypot?"

### Drop-off risk
**Medium.** Morgan actively follows privacy channels — they won't miss it. But they'll dismiss it instantly if the initial post is marketing-heavy with no technical substance.

### What we need
- Technical post explaining the protocol: escrow model, key derivation, proof system, trust assumptions
- Link to open-source Noir circuits and smart contracts
- Honest statement of trade-offs: "OGBank holds the spending key — here's why, here's the roadmap to multi-sig"
- No marketing fluff

---

## Stage 2: Due Diligence

> "Let me verify the math and the trust model."

### What happens
Morgan spends 30-60 minutes reviewing the protocol. They don't need a 60-second explainer — they need to understand the architecture deeply enough to decide whether to trust it with their ZEC.

### What they review
1. **Trust model** — Who holds the spending key? What's the escrow model? What are the custodial risks?
2. **ZK proof system** — What does the proof assert? What's the circuit? Ultrahonk? Can they inspect it?
3. **Key derivation** — How are viewing keys and spending keys derived? ZIP-32 compliant?
4. **Smart contract code** — Is it on GitHub? Is the verifier correct? Aave V3 integration?
5. **Privacy guarantees** — What's hidden, what's revealed, worst case?
6. **Liquidation mechanics** — What happens if ZEC drops? Who triggers it? What oracle? What penalty?

### What they think
- "The escrow model is custodial — that's a real trade-off, but they're honest about it"
- "Ultrahonk is Aztec's system — I trust the cryptography"
- "I can read the Noir circuits. The proof asserts what they claim."
- "The on-chain proof of obligation creates accountability. Not perfect, but better than trusting a promise."
- "Liquidation mechanism still has open questions — I need to understand the oracle before depositing real money"

### Drop-off risk
**High.** This is the decision point. If code isn't open-source, trust model is unclear, documentation is shallow, or they find a vulnerability — they leave. They also leave if liquidation mechanics are handwaved.

### What we need
- Complete open-source codebase on GitHub
- Detailed protocol spec (not marketing docs)
- Honest trade-off documentation: what's custodial, what mitigations exist, roadmap
- Circuit-level documentation: what the ZK proof asserts, inputs, constraints
- Security model: known risks, attack vectors, mitigations
- Clear liquidation documentation: oracle source, threshold, penalty, who triggers it

### Key metric
**Time on docs / GitHub engagement** — Are Morgan-type users reading technical docs and inspecting code?

---

## Stage 3: Onboarding

> "I'll try it with a small amount first."

### What Morgan does
1. Connects Avalanche wallet (MetaMask)
2. Requests a OGBank escrow account
3. Receives a ZCash deposit address + viewing key
4. **Verifies the escrow address** — checks key derivation, confirms the address is valid
5. Sends a small amount of shielded ZEC from their wallet (Ywallet, Zingo)
6. Confirms deposit via viewing key

### What they see
| Step | UI |
|------|---|
| Connect wallet | "Connect your Avalanche wallet" |
| Request account | "Create OGBank Account" |
| Receive address | Deposit address `z1abc...xyz` with copy + viewing key (exportable) |
| Verify | Key derivation details (expandable) |
| Deposit | "Send ZEC to this address from your ZCash wallet" + QR |
| Confirmation | "Deposit confirmed: X ZEC" with viewing key verification |

### Behind the scenes
| Step | Process |
|------|---------|
| Connect wallet | Standard Web3 connection (Avalanche C-Chain) |
| Request account | Relayer generates new ZCash address from ZIP-32 key tree. Spending key stays with protocol. |
| Receive address | User gets deposit address + viewing key. Viewing key is exportable. |
| Deposit | User sends shielded ZEC to escrow address |
| Confirmation | Browser uses viewing key to scan escrow address and confirm arrival |

### Pain points
- The friction isn't "two-wallet complexity" — Morgan already uses ZCash wallets. The friction is **verifying the escrow address is legitimate** and the key derivation is correct.
- Viewing key must be exportable (not browser-only) — Morgan backs it up independently
- First deposit will be small (test amount). Morgan doesn't trust any protocol with real money on day one.

### Drop-off risk
**Low-Medium.** Morgan has the technical skills. The risk is if escrow verification is opaque or the viewing key isn't exportable.

### Key metric
**Onboarding completion rate** — % who create an account AND deposit ZEC.

---

## Stage 4: First Borrow

> "Let me see what the proof actually asserts."

### What Morgan does
1. Chooses borrow amount (USDC)
2. Reviews terms: rate, collateral ratio, liquidation price
3. Inspects proof generation (optional: view proof details)
4. Clicks "Borrow"
5. Receives USDC in Avalanche wallet

### What they see
| Step | UI |
|------|---|
| Amount | "How much USDC?" + max amount, collateral ratio |
| Terms | Interest rate (Aave V3 variable), collateral ratio, liquidation price, oracle source |
| Proof details | Expandable: "This proof asserts: >= X ZEC at escrow [hash], sufficient for Y USDC at Z% collateral ratio" |
| Confirm | "Borrow [amount] USDC" |
| Processing | "Generating proof... Submitting to verifier... Borrowing from Aave V3..." |
| Success | "[amount] USDC received" + Avalanche tx hash |

### Behind the scenes
| Step | Process |
|------|---------|
| Amount selection | Browser uses viewing key to check escrow balance, calculates max borrow |
| Proof generation | Browser generates Ultrahonk proof: "There are >= X ZEC at escrow, sufficient for this borrow" |
| Submission | User submits proof + amount to OGBankContract on Avalanche |
| Verification | On-chain Ultrahonk verifier confirms proof |
| Borrow | OGBankContract calls Aave V3: `approval()` → `supply()` → `borrow()` |
| Transfer | USDC sent to user's Avalanche wallet |

### What Morgan thinks after borrowing
- "OK — I now have USDC. My ZEC is locked. The position is live."
- "What's my liquidation price? What rate am I paying?"
- "I need to watch ZEC/USD. If it drops significantly, I'm in trouble."

### Pain points
- Proof generation takes 10-30 seconds — Morgan understands why but the wait creates anxiety
- If proof fails (browser compatibility, circuit bug), error messages must be technical, not "something went wrong"

### Drop-off risk
**Low.** Morgan is committed at this point. Risk is only technical failure.

### Key metric
**Borrow completion rate** — % of depositors who complete first borrow.

---

## Stage 5: Active Position Management

> "My position is live. I need to manage it."

This is where the real DeFi experience begins. Morgan isn't passively "monitoring" — they're actively managing a leveraged position with cross-chain collateral. This stage can last days, weeks, or months.

### What Morgan sees (Dashboard)
| Element | Display |
|---------|---------|
| Escrow balance | ZEC in escrow (verified by viewing key) |
| Debt | USDC owed (principal + accrued interest) |
| Collateral ratio | Current %, updated with ZEC/USD price |
| Liquidation price | Exact ZEC/USD price at which liquidation triggers |
| Interest rate | Current Aave V3 variable rate + accrued interest to date |
| Oracle feed | Source, last update, price history |
| Verification | "Verify escrow balance" button (triggers viewing key scan) |
| Actions | Repay (full/partial), claim (after full repay) |
| History | All transactions with on-chain links |

### Routine: stable market
Morgan checks in every few days. ZEC/USD is stable. Collateral ratio stays comfortable (>200%). Interest accrues slowly. Nothing to do.

**What they think:** "Position is healthy. Rates are reasonable. I'll keep earning yield with my USDC on Aave."

### Scenario: Interest rate spike
Aave V3 variable rates spike (e.g., high USDC utilization). Morgan's cost of holding the position increases.

**What they think:** "Rate went from 3% to 12%. Is it worth keeping this position open? How much interest am I accruing daily?"

**What they do:**
- Check current rate vs. what they're earning with the borrowed USDC
- If the math stops working, consider partial or full repayment
- Need: clear display of daily interest accrual, not just APY

### Scenario: ZEC appreciates
ZEC/USD goes up 30%. Morgan's collateral ratio improves significantly.

**What they think:** "My collateral ratio went from 180% to 234%. Can I borrow more USDC against the same collateral?"

**What they do:**
- Want to borrow additional USDC (top-up borrow)
- Need: ability to generate a new proof with updated collateral value and borrow more
- **Open question:** Does the protocol support incremental borrows against existing collateral, or only one borrow per cycle?

### Scenario: ZEC depreciates (pre-critical)
ZEC/USD drops 15%. Collateral ratio goes from 180% to 153%.

**What they think:** "I'm still above liquidation (120%) but the buffer is shrinking. Do I act now or wait?"

**What they do:**
- Check the oracle: is the price drop real or a temporary wick?
- Calculate: at what price does liquidation trigger?
- Consider options:
  1. **Partial repay** — return some USDC to reduce debt and improve ratio
  2. **Add collateral** — deposit more ZEC to the escrow to increase collateral value
  3. **Wait** — if they believe ZEC will recover
- Need: clear "what-if" scenarios — "If you repay X USDC, your new ratio is Y% and liquidation price is Z"

**Open question:** Can users add more ZEC to an existing escrow position? This requires a new deposit to the same escrow address + an updated proof. Protocol must support this.

### What we need
- Raw numbers: ratio %, liquidation price, oracle source and freshness, daily interest accrual
- Independent escrow verification via viewing key
- Oracle transparency: which feed, last update, staleness detection
- Partial repayment support
- Clear "what happens if ZEC goes to $X" simulator
- Notifications/alerts at configurable thresholds (not just at critical levels)

### Key metric
**30-day retention** — % of borrowers with an active position (or a successfully closed one) after 30 days.

---

## Stage 6: Undercollateralization

> "ZEC is crashing. I need to decide now."

This is not a footnote. This is the highest-stress moment in the entire journey. How we handle it determines whether Morgan ever uses OGBank again — and what they tell the ZCash community.

### Trigger
ZEC/USD drops enough that Morgan's collateral ratio approaches the liquidation threshold (120%).

### Timeline of a typical undercollateralization event

**Warning zone (ratio 140-150%)**
- Dashboard shows amber alert: "Collateral ratio at 142%. Liquidation at 120%."
- Morgan has time to act. This is where they decide.

**Decision moment:**

| Option | Action | Result |
|--------|--------|--------|
| Partial repay | Return some USDC to reduce debt | Ratio improves immediately |
| Add collateral | Deposit more ZEC to escrow | Ratio improves after deposit confirms (~75s ZCash block) |
| Full repay + claim | Close the position entirely | Exits risk, gets ZEC back (at lower value) |
| Do nothing | Wait for price recovery | Either recovers or gets liquidated |

**Critical zone (ratio 120-130%)**
- Dashboard shows red alert: "Collateral ratio at 125%. Liquidation imminent."
- If Morgan chose to act, they need to act NOW
- Partial repayment is the fastest option (on-chain on Avalanche, ~2 seconds)
- Adding collateral is slower (ZCash block time ~75 seconds + viewing key scan + new proof)

**Liquidation triggered (ratio < 120%)**
- See Stage 8: Liquidation

### What Morgan thinks
- "I should have set up alerts earlier."
- "Can I repay fast enough? The USDC is in a yield position on Aave — I need to withdraw it first."
- "Adding collateral requires a ZCash transaction — that's 75 seconds I might not have."
- "If I repay partially, what's my new ratio? Is it enough?"

### Pain points
- **Speed matters.** In a crash, ZEC can move 5-10% in minutes. The user needs to execute decisions fast.
- **Cross-chain friction.** Adding collateral requires a ZCash transaction, then waiting for confirmation, then generating a new proof. This can take 2-5 minutes — an eternity during a crash.
- **Cascading complexity.** If the borrowed USDC is deployed in other DeFi protocols (Aave yield, LP positions), Morgan needs to unwind those positions first before repaying. We don't control that.
- **Oracle lag.** If the oracle price feed is delayed during high volatility, Morgan might get liquidated at a stale price.

### What we need
- Configurable alerts (email, webhook, Telegram) at user-defined thresholds
- One-click partial repayment with "what-if" preview
- Clear display of: current ratio → ratio after repayment → liquidation price
- If adding collateral is supported: display expected time to confirmation
- Oracle staleness indicator (time since last update, deviation from market)

---

## Stage 7: Return & Claim

> "USDC returned. Let me verify the ZEC release."

### What Morgan does
1. Repays borrowed USDC (plus accrued interest)
2. Generates repayment proof
3. Submits claim to OGBankContract
4. Monitors on-chain event emission
5. Verifies ZEC arrival at their ZCash address via viewing key

### What they see
| Step | UI |
|------|---|
| Repay | "Repay [principal + interest] USDC" with breakdown |
| Confirmation | "Loan fully repaid" + Avalanche tx hash |
| Claim | "Claim your ZEC" |
| Proof details | Expandable: "This proof references borrow nullifier [hash] and asserts full repayment of [amount] USDC" |
| Processing | "Generating repayment proof... Verifying on-chain... Signaling relayer..." |
| Event | "FinishPayment event emitted. Relayer signaled to return ZEC." + tx hash |
| ZEC status | "ZEC return pending... Confirmed in block [number]" |
| Done | "Your ZEC has been returned to your ZCash address" |

### Behind the scenes
| Step | Process |
|------|---------|
| Repay | USDC → OGBankContract → `Repay(amount)` → forwarded to Aave V3 |
| Repayment proof | Browser generates proof referencing borrow nullifier: "This specific loan has been fully repaid" |
| Verification | Proof submitted to OGBankContract. Ultrahonk verifier confirms. Nullifier checked and consumed. |
| Release signal | Contract emits `FinishPayment(ogbank, amount, recipient, address)` event |
| ZEC return | Relayer detects event. Sends ZEC from escrow back to user's original ZCash address. |

### Pain points
- Interest accrual means the repayment amount isn't exactly what Morgan borrowed. The exact payoff amount must be clear (not a surprise).
- ZEC return depends on the relayer. If relayer is down, Morgan has repaid but can't get their ZEC. The on-chain `FinishPayment` event is their proof of obligation.
- ZCash block confirmation takes ~75 seconds. Morgan will check independently (viewing key scan) rather than trust the UI.

### Drop-off risk
**Low.** Morgan is motivated to get their ZEC back. Concern is verifying the return, not deciding whether to do it.

### What we need
- Exact payoff amount (principal + accrued interest) before confirmation
- On-chain event hash visible for independent verification
- ZEC return status with ZCash block confirmations
- Viewing key verification: scan escrow address to confirm it's now empty
- If return is delayed: clear status, expected time, and proof of obligation (the on-chain event)

### Key metric
**Claim success rate** — % of repaid loans where ZEC is returned. Must be 100%.

---

## Stage 8: Liquidation

> "I got liquidated. Was it fair?"

### Trigger
ZEC/USD drops below the liquidation threshold. Morgan either couldn't act in time, chose not to, or the drop was too fast.

### What Morgan sees

| Phase | Display |
|-------|---------|
| **Warning** | "Collateral ratio at 125%. Liquidation at 120%. Consider repaying." |
| **Critical** | "Collateral ratio at 121%. Liquidation imminent." |
| **Liquidated** | "Position liquidated. X ZEC seized to cover Y USDC debt. Penalty: Z%. Remaining ZEC: W." |
| **Post-liquidation** | Closed position breakdown: ZEC seized, price at liquidation, penalty, remaining balance. "Claim remaining ZEC" if any. |

### What Morgan does after
1. Checks on-chain events: was the liquidation triggered at the correct price?
2. Verifies oracle feed: was the price accurate? Was it stale?
3. Checks remaining ZEC in escrow via viewing key
4. If fair, accepts it. If unfair (oracle manipulation, wrong threshold, penalty mismatch), reports to ZCash community.

### Open design questions (affect UX)
These are acknowledged unknowns from the protocol design. They directly impact the liquidation experience:

| Question | Impact on UX |
|----------|-------------|
| Who triggers liquidation? Protocol-automated or external liquidators? | Automated = faster, less MEV risk. External = permissionless but introduces liquidation bots and potential front-running. |
| How is seized ZEC converted to USDC to repay Aave? | OTC? DEX on another chain? Bridge and sell? This affects the liquidation penalty and speed. |
| What's the liquidation penalty? | Standard DeFi: 5-15%. Higher penalty = more safety buffer for the protocol, more loss for the user. |
| Partial or full liquidation? | Partial = liquidate only enough to restore ratio. Full = seize everything. Partial is better UX but more complex. |
| What happens if ZEC value drops below total debt? | Bad debt. Protocol absorbs the loss. Or: socialized across all depositors? This is existential. |

### Design implications
1. **Full transparency** — Morgan verifies, not just reads. Every liquidation must emit events with: trigger price, oracle source, ZEC seized, penalty, remaining balance.
2. **Oracle reliability is existential** — One bad liquidation based on a stale/manipulated oracle, and Morgan tells the entire ZCash community. Chainlink minimum. DEX spot prices are flash-loan vulnerable.
3. **Prevention > cure** — The warning system (Stage 6) matters more than the liquidation UX itself. If Morgan gets liquidated without adequate warning, that's a product failure regardless of how clean the liquidation was.

---

## Stage 9: Re-engagement

> "That worked. Let me do it again."

### Happy path re-engagement
Morgan closed a position successfully. ZEC returned. Cycle complete. They now:
- Deposit again (possibly more ZEC this time — trust is built)
- Borrow with better knowledge of rates, timing, and position management
- Potentially hold a longer-term position

### Post-liquidation re-engagement
Morgan got liquidated but verified it was fair. They:
- Deposit again with a more conservative collateral ratio
- Set up alerts they didn't have before
- Borrow less relative to collateral

**If the liquidation was unfair or unverifiable, Morgan doesn't come back. And they tell the community.**

### What drives re-engagement
- Yield opportunity still exists (ZEC is still locked out of DeFi)
- Protocol proved reliable in the first cycle
- Trust builds: "my ZEC was returned correctly" is the strongest signal

### What kills re-engagement
- ZEC wasn't returned (or was delayed without explanation)
- Liquidation felt unfair (oracle issue, excessive penalty, no transparency)
- Interest rate made the position unprofitable
- UX friction that wasn't worth the yield

---

## Failure Scenarios Across the Journey

These aren't edge cases. They're things that will happen.

| Scenario | Stage affected | User impact | What Morgan does |
|----------|---------------|-------------|-----------------|
| **Relayer down** | 7 (Claim) | Repaid loan but can't get ZEC back | Waits. Has on-chain proof of obligation (`FinishPayment` event). If prolonged, escalates to community. |
| **Proof generation fails** | 4, 7 | Can't borrow or claim | Retries. Checks browser (WebAssembly required). Fallback: CLI proof generation tool. |
| **Aave V3 paused** | 4, 5, 6 | Can't borrow. Can't repay. Can't escape a declining position. | Waits for Aave to resume. ZEC safe in escrow but position is frozen — interest may still accrue. |
| **Oracle stale during crash** | 6, 8 | Liquidated at stale price or unable to assess real ratio | Verifies post-hoc. Reports if liquidation was at wrong price. |
| **ZCash network congested** | 3, 6 | Deposit or additional collateral delayed | Waits for ZCash block. Adding collateral during a crash is time-critical — congestion makes it worse. |
| **Viewing key lost** | All post-3 | Can't generate proofs, can't verify, can't claim | Recovery via Kohaku wallet backup or relayer re-derivation (if user proves identity via `user_secret`). |
| **ZEC flash crash** | 6, 8 | Positions go underwater before anyone can react | Liquidation triggers. Possible bad debt if ZEC drops below debt value. Over-collateralization buffer is the only defense. |
| **Smart contract bug** | 4, 7, 8 | Funds at risk | Emergency pause. This is why audit + testnet phase + bug bounty matter. |

---

## Journey Summary

| Stage | What Morgan does | Decision | Biggest risk |
|-------|-----------------|----------|-------------|
| 1. Discovery | Finds OGBank via ZCash channels | "Is this worth investigating?" | Too marketing-heavy, no substance |
| 2. Due diligence | Reviews code, architecture, trust model | "Do I trust this with my ZEC?" | Code not open, trust model vague, liquidation mechanics unclear |
| 3. Onboarding | Deposits small test amount | "Is this escrow address legit?" | Verification opaque, viewing key not exportable |
| 4. First borrow | Borrows USDC, inspects proof | "Does the proof assert what they claim?" | Proof failure, unclear assertions |
| 5. Active management | Monitors position, reacts to market | "Is this position still worth it?" | Stale oracle, hidden rate changes, no alerts |
| 6. Undercollateralization | Decides: repay, add collateral, wait | "Can I fix this in time?" | Cross-chain latency, oracle lag, cascading DeFi positions |
| 7. Return & claim | Repays, verifies ZEC return | "Did I get my ZEC back?" | Relayer down, delayed return |
| 8. Liquidation | Verifies fairness post-hoc | "Was this fair?" | Oracle manipulation, wrong threshold, no transparency |
| 9. Re-engagement | Deposits again (or doesn't) | "Was the experience worth repeating?" | Broken trust from any previous stage |

---

## Phase 2 Journey: Alex (The Private Holder)

> Future reference only. NOT the MVP design target.

Alex is crypto-curious, holds ZCash, wants to borrow privately but isn't DeFi-native. Arrives after Phase 1 social proof ("used by the ZCash community").

### Key differences from Morgan

| Stage | Morgan (Phase 1) | Alex (Phase 2) |
|-------|------------------|----------------|
| **Discovery** | ZCash Forum, Discord, GitHub | Crypto Twitter, SEO, Reddit |
| **Due diligence** | 30-60 min deep technical review | 60-second landing page, visual explainer |
| **Onboarding** | Verifies escrow and key derivation | Two-wallet complexity, never sent shielded ZEC |
| **First borrow** | "What does the proof assert?" | "Why is it taking so long?" |
| **Active management** | Raw numbers, oracle source, independent verification | Simplified health factor, proactive alerts |
| **Undercollateralization** | Calculates options, acts rationally | Panics, doesn't know what "collateral ratio" means |
| **Return & claim** | Verifies ZEC return independently | Trusts UI confirmation |
| **Liquidation** | Verifies fairness on-chain | Feels betrayed — didn't understand the risk |

### What Alex needs that Morgan doesn't
- Zero jargon on landing page
- Visual explainer (video/infographic)
- Step-by-step onboarding with wallet-specific instructions
- Simplified health display (not raw ratio)
- Educational: "What happens if ZEC drops?" (before it happens)
- Trust signals: community usage, Aave V3 badge, audit reports

### When to build for Alex
After Phase 2 Gate (see [09_metrics.md](09_metrics.md)):
- 100+ deposits from ZCash-native users
- 0 collateral loss events
- Completed security audit
- Positive ZCash community sentiment

---

## Links

- Who Morgan is: [05_user-persona.md](05_user-persona.md)
- Landing page spec: [10_landing.md](10_landing.md)
- Metrics framework: [09_metrics.md](09_metrics.md)
- Product architecture: [04_architecture.md](04_architecture.md)
- Protocol spec: [../technical/02_protocol.md](../technical/02_protocol.md)

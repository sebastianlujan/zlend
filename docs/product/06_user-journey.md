# User Journey

This journey follows **Morgan (The ZCash Native)** — our launch persona. A technically proficient ZCash holder who has been waiting for DeFi access without breaking privacy.

Alex's journey (Phase 2, crypto-curious) is included at the end as a future reference. It is NOT the design target for MVP.

---

## Stage 1: Awareness

> "Finally — someone is building DeFi access for ZCash without breaking privacy."

### What Happens
Morgan sees a technical post on the ZCash Community Forum, a mention in ZCash Discord, or a talk at Zcon about "locking shielded ZEC to unlock liquidity on Avalanche — privacy preserved." They've been waiting for something like this. They've probably already wrapped ZEC on Solana or BSC reluctantly and want a better option.

### Channels
- ZCash Community Forum (primary)
- ZCash Discord
- Zcon / privacy-focused developer conferences
- GitHub (open-source code drops)
- Word of mouth from other ZCash-native developers

### What They Think
- "How does the escrow work? Who holds the spending key?"
- "Which ZK proving system? Is it audited?"
- "Can I verify the circuits myself?"
- "Is this another custodial honeypot?"

### Landing Site Touchpoint — Home Page (`/`)
Morgan clicks through to the landing site. The Home page tells the story in 3 sections:

1. **Hero** — "Access DeFi. Keep your ZEC." Immediate clarity. Two CTAs: "Go to App" (placeholder until app is live) and "Read the Docs" (→ Technology page).
2. **Problem** — "Your ZEC Has Privacy. It Has Almost Nothing Else." Morgan sees the opportunity gap: ZEC has strong foundations but can't access yield, swaps, farming. The side-by-side (ZEC Alone vs ZEC + Stables) makes the case visually.
3. **Solution** — "Lock. Unlock. Access." Split-world diagram: Private World (ZCash) ↔ OGBank ↔ DeFi World (Avalanche). Three steps with network badges. Morgan understands the architecture at a glance.

The cypherpunk terminal aesthetic (monospace, `//` tags, border-color hovers) signals that this is built by technical people for technical people. No marketing fluff — Morgan stays.

### Drop-Off Risk
**Medium.** Morgan doesn't scroll past — they actively follow privacy-focused channels. But they will dismiss it immediately if the initial post is marketing-heavy with no technical substance. The landing site's terminal aesthetic and lock/unlock framing (not "lending/borrowing") helps retain Morgan.

### What We Need
- Technical post explaining the protocol: escrow model, key derivation, proof system, trust assumptions
- Link to open-source Noir circuits and smart contracts
- Honest statement of trade-offs: "OGBank holds the spending key — here's why, and here's the roadmap to multi-sig"
- No marketing fluff. Morgan filters for signal, not hype.

---

## Stage 2: Due Diligence

> "Let me verify the math and the trust model."

### What Happens
Morgan spends 30-60 minutes reviewing the protocol. They don't need a 60-second explainer — they need to understand the architecture deeply enough to decide whether to trust it with their ZEC.

### What They Review
1. **Trust model** — Who holds the spending key? What's the escrow model? What are the custodial risks?
2. **ZK proof system** — What does the proof assert? What's the circuit? Is it Ultrahonk? Can they inspect it?
3. **Key derivation** — How are viewing keys and spending keys derived? Is it ZIP-32 compliant?
4. **Smart contract code** — Is it on GitHub? Is the verifier correct? How does it integrate with Aave V3?
5. **Privacy guarantees** — What's hidden, what's revealed, what's the worst case?

### Landing Site Touchpoints — Technology + Market Pages

Morgan's due diligence maps directly to the landing site's 3-page structure:

**Technology page** (`/technology`) — answers questions 2-4:
- **HowItWorks section**: 4-step technical pipeline (Deposit ZEC → Generate Proof → Verify On-Chain → Borrow USDC). Each step is an expandable LayerCard showing terminal commands (`zcash-cli z_sendmany ...`, `nargo prove --circuit deposit.nr`). Morgan clicks to verify the architecture matches their mental model.
- **Features section**: Noir + Ultrahonk, Aave V3 Integration, ZIP-32 Key Derivation, Nullifier Protection. Name-drops technologies Morgan already trusts. Hover/tap reveals the detail.

**Market page** (`/market`) — answers questions 1 and 5:
- **MarketData section**: 5.1M ZEC in shielded pools, $0 DeFi access. Validates the opportunity and shows OGBank understands the market.
- **Trust section**: Two-key custody model (Viewing Key = you, Spending Key = protocol). Privacy summary table: what's hidden, what's public, who knows what. Morgan reads this as an honest trade-off disclosure, not marketing.

**GitHub** (external) — answers question 4:
- Open-source Noir circuits, smart contracts, relayer code.

### What They See
- Landing site: Technology page (architecture pipeline, proven primitives) + Market page (trust model, privacy table)
- Open-source code (GitHub repo with contracts, circuits, relayer)
- Trust model: spending key (protocol) vs. viewing key (user) — on the Market page
- Privacy summary table: what's hidden, what's public, who knows what — on the Market page

### What They Think
- "The escrow model is custodial — that's a real trade-off, but they're honest about it"
- "Ultrahonk is Aztec's system — I trust the cryptography"
- "I can read the Noir circuits. The proof asserts what they claim."
- "The on-chain proof of obligation creates accountability. Not perfect, but better than trusting a promise."

### Drop-Off Risk
**Medium-High.** This is where Morgan decides whether to proceed. If the code isn't open-source, the trust model is unclear, or the documentation is shallow, they leave. They will also leave if they find a vulnerability or inconsistency.

### What We Need
- Complete, open-source codebase on GitHub
- Detailed protocol specification (not marketing docs)
- Honest trade-off documentation: what's custodial, what mitigations exist, what the roadmap is
- Circuit-level documentation: what the ZK proof asserts, what inputs it takes
- Security model: known risks, attack vectors, mitigations

### Key Metric
**Time on docs / GitHub engagement** — Are Morgan-type users reading the technical docs and inspecting the code? Landing site: pages/session (does Morgan visit all 3 pages?), time on Technology page.

---

## Stage 3: Onboarding

> "I'll try it with a small amount first."

### What the User Does
1. Connects their Avalanche wallet (MetaMask or similar)
2. Requests a OGBank escrow account
3. Receives a ZCash deposit address + viewing key
4. **Verifies the escrow address** — checks key derivation, confirms the address is valid on ZCash
5. Sends a small amount of shielded ZEC from their wallet (Ywallet, Zingo)
6. Confirms deposit via viewing key

### What They See
| Step | UI Element |
|------|-----------|
| Connect wallet | "Connect your Avalanche wallet" button |
| Request account | "Create OGBank Account" button |
| Receive address | "Your deposit address: z1abc...xyz" with copy button + "Your viewing key" (exportable, not just stored in browser) |
| Verify | Key derivation details visible (optional expandable section) |
| Deposit | "Send ZEC to this address from your ZCash wallet" with copy/QR |
| Confirmation | "Deposit confirmed: X ZEC" with viewing key verification |

### What Happens Behind the Scenes
| Step | Technical Process |
|------|------------------|
| Connect wallet | Standard Web3 wallet connection (Avalanche C-Chain) |
| Request account | Relayer generates a new ZCash address from its key tree (ZIP-32). Creates escrow address with spending key (kept by protocol). |
| Receive address | User gets the deposit address + viewing key. Viewing key exportable for independent verification. |
| Deposit | User sends shielded ZEC from their wallet to the escrow address |
| Confirmation | User's browser uses viewing key to scan the escrow address and confirm the deposit arrived |

### Drop-Off Risk
**Low-Medium.** Morgan already has a ZCash wallet (Ywallet, Zingo) and knows how to send shielded ZEC. The friction is NOT "two-wallet complexity" — it's verifying that the escrow address is legitimate and the key derivation is correct. Morgan wants to be sure before depositing.

### What We Need
- Viewing key must be exportable (not just browser-stored) — Morgan wants to back it up independently
- Optional: show key derivation path for verification
- Compatible with Ywallet and Zingo for sending shielded ZEC
- Clear confirmation when ZEC arrives (viewing key scan)
- Start with testnet flow — Morgan will test before committing real ZEC

### Key Metric
**Onboarding completion rate** — % of users who create an account AND successfully deposit ZEC.

---

## Stage 4: First Borrow

> "Let me see what the proof actually asserts."

### What the User Does
1. Chooses how much to borrow (USDC)
2. Reviews terms (rate, collateral ratio, liquidation level)
3. Inspects the proof generation (optional: view proof details)
4. Clicks "Borrow"
5. Receives USDC in their Avalanche wallet

### What They See
| Step | UI Element |
|------|-----------|
| Amount | Input: "How much USDC do you want to borrow?" with max amount and collateral ratio shown |
| Terms | Detailed card: interest rate (from Aave V3), collateral ratio, exact liquidation price, oracle feed source |
| Proof details | Expandable: "This proof asserts: there are >= X ZEC at escrow address [hash], sufficient for Y USDC borrow at Z% collateral ratio" |
| Confirm | "Borrow [amount] USDC" button |
| Processing | Progress: "Generating Ultrahonk proof... Submitting to verifier... Borrowing from Aave V3..." |
| Success | "You received [amount] USDC" with Avalanche transaction hash (clickable) |

### What Happens Behind the Scenes
| Step | Technical Process |
|------|------------------|
| Amount selection | Browser uses viewing key to check escrow balance, calculates max borrow based on collateral value and ratio |
| Proof generation | Browser generates Ultrahonk ZK proof using viewing key: "There are >= X ZEC at the escrow address, sufficient for this borrow amount" |
| Submission | User submits proof + borrow amount as a transaction to OGBankContract on Avalanche |
| Verification | Ultrahonk Verifier on-chain confirms proof validity |
| Borrow execution | OGBankContract calls Aave V3: `approval()` -> `supply()` -> `borrow()` |
| Token transfer | USDC sent to user's Avalanche wallet |

### Drop-Off Risk
**Low.** Morgan is comfortable with proof generation taking 10-30 seconds — they understand the computational cost. The risk is if the proof fails (browser compatibility, circuit bug) or if the on-chain verification fails unexpectedly.

### What We Need
- Proof details visible (not hidden) — Morgan wants to know what's being asserted
- Transaction hash immediately after submission
- Clear error messaging if proof generation fails (with technical details, not just "something went wrong")
- Exact collateral ratio, liquidation price, and oracle source — not simplified "health factor"

### Key Metric
**First borrow completion rate** — % of users with deposits who complete their first borrow.

---

## Stage 5: Active Use

> "Let me monitor my position and check the oracle."

### What the User Does
- Monitors their position (collateral value vs. debt) with raw numbers
- Checks the ZEC/USD oracle feed directly
- Verifies escrow balance independently via viewing key
- Optionally: partially repays

### What They See
| Element | Display |
|---------|---------|
| Dashboard | ZEC in escrow (verified by viewing key), USDC debt, collateral ratio (%), exact liquidation price in USD, current ZEC/USD from oracle |
| Oracle feed | Source, last update time, price history |
| Verification | "Verify escrow balance" button (triggers viewing key scan) |
| Alerts | "Your collateral ratio is at 135% — liquidation at 120%" |
| Actions | "Repay" button, "Add collateral" (if supported) |
| History | All transactions with on-chain links |

### What Happens Behind the Scenes
- Price oracle (ZEC/USD) tracks collateralization ratio
- Collateral ratio calculated: (ZEC value in escrow) / (USDC debt)
- Browser can independently verify escrow balance via viewing key at any time
- Alert system monitors ratio against liquidation threshold

### Drop-Off Risk
**Low** during normal conditions. During market volatility, Morgan doesn't panic — they monitor the oracle and make rational decisions. The risk is if the oracle feed is stale, unreliable, or manipulable.

### What We Need
- Raw numbers: collateral ratio %, exact liquidation price, oracle source and freshness
- Independent escrow verification via viewing key (not just trust the dashboard)
- Oracle source transparency: which feed, when last updated, what happens if it goes stale
- One-click repay action for quick position management

### Key Metric
**30-day retention** — % of borrowers who still have an active position (or completed one successfully) after 30 days.

---

## Stage 6: Return & Claim

> "USDC returned. Let me verify the ZEC release independently."

### What the User Does
1. Repays the borrowed USDC (plus interest)
2. Generates repayment proof
3. Submits claim to the OGBank contract
4. Monitors on-chain event emission
5. Verifies ZEC arrival at their original ZCash address independently

### What They See
| Step | UI Element |
|------|-----------|
| Repay | "Repay [amount + interest] USDC" button |
| Confirmation | "Loan fully repaid" with Avalanche tx hash |
| Claim | "Claim your ZEC" button |
| Proof details | Expandable: "This proof references borrow nullifier [hash] and asserts full repayment of [amount] USDC" |
| Processing | "Generating repayment proof... Verifying on-chain... Signaling relayer..." |
| Event | "FinishPayment event emitted. Relayer signaled to return ZEC." with tx hash |
| ZEC status | "ZEC return pending... Confirmed in block [number]" |
| Success | "Your ZEC has been returned to your ZCash address" |

### What Happens Behind the Scenes
| Step | Technical Process |
|------|------------------|
| Repay | User sends USDC -> `Repay(amount)` -> forwarded to Aave V3 |
| Repayment proof | Browser generates proof referencing the borrow nullifier: "This specific loan has been fully repaid" |
| Verification | User submits proof to OGBankContract. Ultrahonk Verifier confirms. Contract checks: nullifier exists, not already consumed. |
| Release signal | Contract marks nullifier as consumed. Emits `FinishPayment` event. |
| ZEC return | Relayer detects the on-chain event and sends ZEC from the escrow address back to the user's original ZCash address. |

### Drop-Off Risk
**Low.** Morgan is motivated to get their ZEC back. Their concern is verifying the return happened correctly — they will check the ZCash blockchain independently, not just trust the UI.

### What We Need
- On-chain event hash visible (so Morgan can verify independently on a block explorer)
- ZEC return status with ZCash block confirmation
- Viewing key verification: Morgan can scan the escrow address to confirm it's now empty
- If ZEC return is delayed (ZCash block time ~75 seconds), show clear status with expected confirmation time

### Key Metric
**Successful claim rate** — % of repaid loans where ZEC is successfully returned. Must be 100%.

---

## Scenario: Liquidation

> "ZEC price dropped. Let me verify the liquidation was fair."

### What Triggers This
ZEC/USD price drops below the collateralization threshold. The position becomes undercollateralized.

### What Morgan Sees

| Stage | What Happens |
|-------|-------------|
| **Warning** | Dashboard alert: "Your collateral ratio is at 125% — liquidation threshold is 120%. Consider repaying some USDC." |
| **Critical** | Dashboard warning: "Collateral ratio at 121%. Liquidation imminent." |
| **Liquidated** | Notification: "Your position has been liquidated. X ZEC was seized to cover Y USDC debt. Liquidation penalty: Z%. Remaining ZEC: W." |
| **Post-liquidation** | Dashboard shows closed position with full breakdown: ZEC seized, price at liquidation, penalty applied, remaining balance. If remaining ZEC exists, "Claim remaining ZEC" button available. |

### What Morgan Does After Liquidation
- Checks the on-chain events: was the liquidation triggered at the correct price?
- Verifies the oracle feed: was the price accurate at the time of liquidation?
- Checks remaining ZEC in escrow via viewing key
- If the liquidation was fair, accepts it. If it was unfair (oracle manipulation, incorrect threshold), reports it to the ZCash community.

### Design Implications
1. **Full transparency on liquidation mechanics** — Morgan needs to verify, not just be told
2. **On-chain event trail** — Every liquidation must emit events with: trigger price, oracle source, ZEC seized, penalty, remaining balance
3. **Oracle reliability is existential** — If the oracle is wrong during a liquidation, Morgan tells the entire ZCash community. One bad liquidation can kill Phase 1 adoption.
4. **Proactive warnings** are more important than the liquidation UX itself. Prevent the experience.

---

## Journey Summary (Morgan — Phase 1)

| Stage | User Action | Key Metric | Biggest Risk |
|-------|------------|-----------|-------------|
| 1. Awareness | Discovers OGBank via ZCash channels → visits Home page | Forum/Discord engagement, landing sessions | Too marketing-heavy, not enough technical substance |
| 2. Due Diligence | Reviews Technology + Market pages, inspects code on GitHub | Time on docs, pages/session, GitHub activity | Code not open-source, trust model unclear |
| 3. Onboarding | Deposits ZEC (small test amount first) | Onboarding completion | Escrow address verification, viewing key export |
| 4. First Borrow | Borrows USDC, inspects proof | Borrow completion rate | Proof generation failure, unclear assertions |
| 5. Active Use | Monitors with raw data, verifies independently | 30-day retention | Stale oracle, unreliable data |
| 6. Claim | Repays and verifies ZEC return independently | Claim success rate (must be 100%) | ZEC return delay, unverifiable return |
| 7. Liquidation | Verifies liquidation was fair | Liquidation accuracy | Oracle manipulation, incorrect threshold |

---

## Phase 2 Journey: Alex (The Private Holder)

> This journey applies after Phase 1 validation. It informs future UX investment but is NOT the design target for MVP.

Alex is a crypto-curious user who holds ZCash and wants to borrow privately but is NOT DeFi-native. They arrive when Phase 1 social proof exists ("used by the ZCash community").

### Key Differences from Morgan's Journey

| Stage | Morgan (Phase 1) | Alex (Phase 2) |
|-------|------------------|----------------|
| **Awareness** | ZCash Forum, Discord, GitHub | Crypto Twitter, SEO, Reddit |
| **Education** | 30-60 min deep technical review | 60-second landing page, visual explainer |
| **Onboarding friction** | Verifying escrow and key derivation | Two-wallet complexity, never sent shielded ZEC |
| **First Borrow concern** | "What does the proof assert?" | "Why is it taking so long?" (proof generation wait) |
| **Active Use** | Raw numbers, oracle source, independent verification | Simplified health factor, proactive alerts |
| **Repay & Claim** | Verifies ZEC return on ZCash blockchain independently | Trusts the UI confirmation |
| **Liquidation** | Verifies fairness on-chain | Feels betrayed if they didn't understand the risk |

### What Alex Needs That Morgan Doesn't
- Zero jargon on landing page
- Visual explainer (video or infographic)
- Step-by-step onboarding with QR codes and wallet-specific instructions
- Simplified health factor display (not raw collateral ratio)
- Educational content: "What happens during liquidation?" (pre-emptive, not post-facto)
- Trust signals: "Used by the ZCash community", Aave V3 badge, audit reports

### When to Build for Alex
Only after the Phase 2 Gate is met (see [09_metrics.md](09_metrics.md)):
- 100+ deposits from ZCash-native users
- 0 collateral loss events
- Completed security audit
- Positive ZCash community sentiment

---

## Links

- Who Morgan is: [05_user-persona.md](05_user-persona.md)
- Landing page spec: [10_landing.md](10_landing.md)
- Metrics framework: [09_metrics.md](09_metrics.md)
- How the architecture works: [04_architecture.md](04_architecture.md)
- Technical protocol flow: [../technical/02_protocol.md](../technical/02_protocol.md)

# User Journey

This journey follows **Alex (The Private Holder)** — our primary persona. A crypto-curious user who holds ZCash and wants to borrow against it privately.

---

## Stage 1: Awareness

> "I didn't know private lending existed."

### What Happens
Alex sees a post in a crypto subreddit, a tweet, or a ZCash community discussion about "borrowing against ZEC without revealing your position." They're intrigued because they've been holding ZEC with no way to use it in DeFi.

### Channels
- ZCash community forums and Discord
- Crypto Twitter / Farcaster
- Reddit (r/cryptocurrency, r/zec, r/defi)
- DeFi aggregator listings
- Word of mouth from ZCash-native users (Morgan persona)

### What They Think
- "Wait, you can borrow against ZCash now?"
- "Nobody can see my position? How?"
- "Is this legit?"

### Drop-Off Risk
**High.** Most people scroll past. The concept is novel and might sound too good to be true.

### What We Need
- Clear, jargon-free messaging: "Borrow stablecoins. Keep your ZCash. Stay private."
- Social proof: early users, audits, "built on Aave V3" badge
- One-click path from awareness to education

---

## Stage 2: Education

> "OK, show me how this actually works."

### What Happens
Alex clicks through to the ZLend landing page or docs. They need to understand three things in under 60 seconds:
1. What it does (borrow against ZCash)
2. Why it's different (nobody sees your collateral)
3. Why it's safe (built on Aave, your keys stay with you)

### What They See
- Landing page with the 3-step model: Lock, Prove, Borrow
- "How It Works" section with the user-facing vs behind-the-scenes comparison
- FAQ: "Is my ZEC safe?", "What if ZEC price drops?", "Who can see my position?"
- Trust indicators: Aave V3 badge, audit status, open-source code

### What They Think
- "This seems simpler than I expected"
- "Built on Aave — I've heard of them"
- "My keys never leave my browser — that's good"
- "What happens if I get liquidated?"

### Drop-Off Risk
**Medium.** Complexity kills here. If they see ZK jargon, nullifiers, or key derivation, they leave.

### What We Need
- Zero jargon on the landing page
- Visual explainer (diagram or short video)
- Prominent liquidation explanation (clear, not scary)
- "Try it" CTA that leads to onboarding

---

## Stage 3: Onboarding

> "Let me try this."

### What the User Does
1. Connects their Avalanche wallet (MetaMask or similar)
2. Gets a unique ZLend address (generated in-browser)
3. Sends ZEC to that address from their ZCash wallet

### What They See
| Step | UI Element |
|------|-----------|
| Connect wallet | "Connect your Avalanche wallet" button |
| Generate address | "Your private ZLend address: z1abc...xyz" with copy button |
| Fund | "Send ZEC to this address from your ZCash wallet" with amount input |
| Confirmation | "Collateral received" with balance display |

### What Happens Behind the Scenes
| Step | Technical Process |
|------|------------------|
| Connect wallet | Standard Web3 wallet connection (Avalanche C-Chain) |
| Generate address | ZIP-32 key derivation: `m_Sapling / 32' / 133' / account' / zlend_index`. Browser creates spending key (stored locally) and viewing key. |
| Fund | User sends shielded ZEC to the derived address. Relayer provides nonce and nullifier. |
| Confirmation | `SupplyTransfer(amount, UTk)` and `connectVk(vk)` called on ZLendContract |

### Drop-Off Risk
**High.** This is the hardest step — requires two wallets (ZCash + Avalanche) and a cross-chain transfer. The ZCash-to-Avalanche bridge is the biggest friction point.

### What We Need
- Step-by-step guided flow with progress indicator
- Clear instruction for sending ZEC (compatible with major ZCash wallets: Ywallet, Zingo, etc.)
- Real-time confirmation when ZEC arrives
- Error handling: what to do if ZEC doesn't arrive, timeout guidance

### Key Metric
**Onboarding completion rate** — % of users who connect wallet AND successfully fund collateral.

---

## Stage 4: First Borrow

> "OK, let's do this."

### What the User Does
1. Chooses how much to borrow (stablecoins)
2. Reviews terms (rate, collateral ratio, liquidation level)
3. Clicks "Borrow"
4. Receives stablecoins in their Avalanche wallet

### What They See
| Step | UI Element |
|------|-----------|
| Amount | Slider or input: "How much do you want to borrow?" with max amount shown |
| Terms | Card showing: interest rate, collateral ratio, estimated liquidation price |
| Confirm | "Borrow [amount] USDC" button |
| Processing | Progress bar: "Generating proof... Verifying... Borrowing..." |
| Success | "You received [amount] USDC" with transaction link |

### What Happens Behind the Scenes
| Step | Technical Process |
|------|------------------|
| Amount selection | Client calculates max borrow based on collateral value and ratio |
| Proof generation | Browser runs Noir circuit → Ultrahonk proof. Asserts: "I own ZEC >= threshold, viewing key matches, UTXOs not already used" |
| Submission | Proof sent to relayer → relayer submits `Borrow(proof, amount)` to ZLendContract |
| Verification | Ultrahonk Verifier on-chain confirms proof validity |
| Borrow execution | ZLendContract calls Aave V3: `approval()` → `supply()` → `borrow()` |
| Token transfer | Stablecoins sent to user via ProtoSocolo ERC-20 |

### Drop-Off Risk
**Medium.** Proof generation takes time (could be 10-30 seconds). Users might think it's stuck. The "processing" step needs clear feedback.

### What We Need
- Real-time progress indicator during proof generation
- Clear explanation of what "generating proof" means ("Proving your collateral is sufficient — this takes a few seconds")
- Immediate confirmation with transaction hash
- Clear display of new position: collateral locked, amount borrowed, liquidation level

### Key Metric
**First borrow completion rate** — % of funded users who complete their first borrow.

---

## Stage 5: Active Use

> "I have a loan. Now what?"

### What the User Does
- Monitors their position (collateral value vs. debt)
- Receives alerts if collateral ratio approaches liquidation
- Optionally: borrows more, partially repays, adds collateral

### What They See
| Element | Display |
|---------|---------|
| Dashboard | Current position: collateral value, debt, health factor, liquidation level |
| Alerts | Push notification or email: "Your health factor is approaching 1.2 — consider adding collateral or repaying" |
| Actions | "Borrow more", "Repay", "Add collateral" buttons |
| History | Past transactions (borrows, repayments) — on their Avalanche wallet only |

### What Happens Behind the Scenes
- Price oracle monitors ZEC/USD to track collateralization ratio
- Health factor calculated: if collateral value drops too close to debt, position becomes liquidatable
- Periodic solvency checks may require user to submit a proof (see privacy model)

### Drop-Off Risk
**Low** during normal conditions. **High** during market volatility (panic, confusion about liquidation).

### What We Need
- Clear, non-technical health factor display
- Proactive alerts BEFORE liquidation threshold (not at the moment of liquidation)
- One-click repay and add-collateral actions
- Educational content: "What happens during liquidation?" (transparent, not hidden)

### Key Metric
**30-day retention** — % of borrowers who still have an active position (or completed one successfully) after 30 days.

---

## Stage 6: Repay & Withdraw

> "Done. Give me back my ZEC."

### What the User Does
1. Repays the borrowed amount (plus interest)
2. Requests collateral withdrawal
3. Receives ZEC back in their shielded wallet

### What They See
| Step | UI Element |
|------|-----------|
| Repay | "Repay [amount + interest]" button |
| Confirmation | "Loan fully repaid" |
| Withdraw | "Withdraw collateral" button |
| Processing | "Generating withdrawal proof... Verifying..." |
| Success | "Your ZEC has been released" |

### What Happens Behind the Scenes
| Step | Technical Process |
|------|------------------|
| Repay | User sends stablecoins → `Repay(amount)` → forwarded to Aave V3 |
| Withdrawal proof | Browser generates proof referencing the borrow nullifier. Proves: "This specific loan was fully repaid." |
| Verification | Ultrahonk Verifier confirms proof. Contract checks: nullifier exists, not already consumed. |
| Release | Contract marks nullifier as consumed. Emits `FinishPayment` event. Collateral released to user's shielded ZCash address. |

### Drop-Off Risk
**Low.** User is motivated to get their ZEC back. Main risk: confusion about the withdrawal proof step.

### What We Need
- One-click "Repay and Withdraw" flow (combine both steps if possible)
- Clear confirmation that ZEC is on its way back
- Explanation if withdrawal proof takes time

### Key Metric
**Successful withdrawal rate** — % of repaid loans where collateral is successfully withdrawn.

---

## Journey Summary

| Stage | User Action | Key Metric | Biggest Risk |
|-------|------------|-----------|-------------|
| 1. Awareness | Discovers ZLend | Click-through rate | Sounds too good to be true |
| 2. Education | Understands value | Time on page, bounce rate | Jargon kills interest |
| 3. Onboarding | Funds collateral | Onboarding completion | Two-wallet friction |
| 4. First Borrow | Borrows stablecoins | Borrow completion rate | Proof generation wait time |
| 5. Active Use | Monitors position | 30-day retention | Liquidation confusion |
| 6. Withdrawal | Gets ZEC back | Withdrawal success rate | Withdrawal proof confusion |

---

## Links

- Who Alex is: [05_user-persona.md](05_user-persona.md)
- How the architecture works: [04_architecture.md](04_architecture.md)
- Technical protocol flow: [../02_protocol.md](../02_protocol.md)

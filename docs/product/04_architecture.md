# Architecture — Product View

## How OGBank Works (3 Steps)

```
  1. DEPOSIT             2. PROVE & BORROW        3. REPAY & CLAIM
 ┌──────────────┐      ┌──────────────────┐      ┌──────────────────┐
 │ Send ZEC to  │ ───▶ │ Your browser     │ ───▶ │ Repay USDC on    │
 │ your OGBank   │      │ proves your      │      │ Avalanche. Prove │
 │ escrow       │      │ deposit. Borrow  │      │ repayment. Get   │
 │ address      │      │ USDC on          │      │ your ZEC back.   │
 │              │      │ Avalanche.       │      │                  │
 └──────────────┘      └──────────────────┘      └──────────────────┘
```

---

## What You See vs. What Happens Behind the Scenes

### Requesting an Account

| What You See | What Happens Behind the Scenes |
|-------------|-------------------------------|
| "Create OGBank Account" | The protocol's relayer generates a new ZCash address from its key tree |
| "Your deposit address: z1abc...xyz" | You receive the ZCash address + a **viewing key** so you can monitor the balance |
| | The **spending key** stays with the protocol (like a bank holding escrow keys) |

### Depositing Collateral

| What You See | What Happens Behind the Scenes |
|-------------|-------------------------------|
| "Send ZEC to your deposit address" | You send shielded ZEC from your wallet to the OGBank escrow address |
| "Deposit confirmed: X ZEC" | Your **viewing key** lets you see the balance arrived. The ZEC is now held by the protocol. |

### Borrowing

| What You See | What Happens Behind the Scenes |
|-------------|-------------------------------|
| "Choose borrow amount" | Your browser uses the viewing key to scan the ZCash deposit and calculate available collateral |
| "Generating proof..." | Browser generates a ZK proof: "There are X ZEC at this address" — without revealing the address or exact amount on-chain |
| "Borrow [amount] USDC" button | You submit the proof + desired amount to the OGBank contract on Avalanche |
| Loading spinner | On-chain Ultrahonk verifier checks the proof. If valid, the contract borrows USDC from Aave V3. |
| "You received [amount] USDC" | USDC arrives in your Avalanche wallet |

### Repaying

| What You See | What Happens Behind the Scenes |
|-------------|-------------------------------|
| "Repay [amount] USDC" | Standard ERC-20 transfer of USDC to the OGBank contract |
| "Loan repaid" | OGBank forwards the repayment to Aave V3 |

### Claiming Your ZCash Back

| What You See | What Happens Behind the Scenes |
|-------------|-------------------------------|
| "Claim collateral" | Your browser generates a repayment proof: "This specific loan was fully repaid" |
| "Submitting claim..." | You submit the proof to the OGBank contract on Avalanche |
| "Claim verified" | The contract verifies the proof, marks this loan as completed |
| "ZEC returned" | The protocol's relayer sends your ZEC from the escrow address back to your original ZCash address |

### Liquidation (When Collateral Value Drops)

| What You See | What Happens Behind the Scenes |
|-------------|-------------------------------|
| "Your collateral ratio is 125% — liquidation at 120%" | Price oracle (ZEC/USD) detects declining collateral value. Dashboard updates in real-time. |
| "Collateral ratio critical — liquidation imminent" | Position flagged as at-risk. Two detection mechanisms: (1) **Proof-based:** user must periodically submit a solvency proof (ZK proof that collateral >= threshold). Failure to submit within the window flags the position. (2) **Oracle-based:** OGBankContract checks committed collateral value against borrow amount using the price feed. |
| "Your position has been liquidated. X ZEC seized to cover Y USDC debt. Remaining: Z ZEC." | Protocol seizes ZEC from the escrow address (relayer holds spending key) and sells/converts it to cover the USDC debt on Aave V3. Liquidation penalty applied. |
| "Claim remaining ZEC" (if any) | If collateral value exceeded the debt + penalty, remaining ZEC is returned to the user's ZCash address. |

**Open design questions:**
- Who triggers liquidation? (Protocol-automated via relayer, or open to external liquidators?)
- How is seized ZEC converted to USDC to repay Aave? (OTC? DEX on another chain? Bridge and sell?)
- What is the liquidation penalty? (Standard DeFi is 5-15%)
- How does partial liquidation work? (Liquidate only enough to restore collateral ratio?)

---

## The Trust Model

### Who Holds What

OGBank uses two keys derived from ZCash's ZIP-32 key tree:

| Key | Who Has It | What It Does |
|-----|-----------|-------------|
| **Spending key** | OGBank protocol (relayer) | Can move ZEC in and out of escrow addresses. Required to return collateral after repayment. |
| **Viewing key** | The user (via their wallet) | Can see the balance at the escrow address. Used to generate ZK proofs. Cannot move funds. |

**The viewing key is critical.** It's the only way to prove you deposited ZEC and claim it back. OGBank does not store viewing keys — they live in the user's wallet, managed by Kohaku (Ethereum Foundation's privacy wallet SDK). This means the user's wallet handles backup and security, not OGBank's servers.

**Think of it like a bank escrow:** You deposit your valuables into a vault. The bank holds the vault key. You hold a window to see inside (viewing key). When the terms of the agreement are met (loan repaid + proof verified), the bank opens the vault and returns your assets.

### The Relayer: A Trusted Escrow

The OGBank relayer manages the ZCash side of the protocol. It holds spending keys for all escrow addresses and executes ZEC transfers.

**What the relayer CAN do:**
- Create new ZCash escrow addresses for users
- See balances at escrow addresses (it holds the spending key)
- Move ZEC from escrow addresses (return collateral after repayment)

**What the relayer CANNOT do:**
- Forge a ZK proof (mathematically impossible — only viewing key holders can prove deposits)
- Skip the on-chain verification (the Avalanche contract enforces proof checks)
- Connect your ZCash identity to your Avalanche identity (the protocol acts as an intermediary)

**What about custodial risk?**

This is the honest trade-off: OGBank holds your ZEC during the loan period. The relayer technically *could* not return your ZEC. We mitigate this through:

1. **On-chain proof of obligation** — When the repayment proof is verified on Avalanche, it creates a public, auditable record that the protocol owes you ZEC
2. **Protocol-level custody** — The relayer is the protocol itself, not a third party. Its reputation and operation depend on returning funds.
3. **Future: multi-sig escrow** — Spending keys can be split across multiple parties using threshold signatures, eliminating single-point-of-failure risk

---

## What's On Which Network

```
┌───────────────────┐          ┌───────────────────────────────┐
│    ZCash Network   │          │    Avalanche C-Chain           │
│                    │          │                               │
│  Escrow address    │          │  OGBank smart contract         │
│  (protocol holds   │ ──────▶ │  (verifies proofs, manages    │
│   spending key)    │   proof  │   loans)                      │
│                    │          │         │                     │
│  User deposits ZEC │          │         ▼                     │
│  User sees balance │          │  Aave V3 lending pool         │
│  via viewing key   │          │  (issues USDC borrows)        │
│                    │          │         │                     │
│                    │  ◀────── │         ▼                     │
│  Protocol returns  │  signal  │  USDC → user's wallet         │
│  ZEC after repay   │          │                               │
└───────────────────┘          └───────────────────────────────┘
```

| Component | Network | What It Does |
|-----------|---------|-------------|
| Escrow addresses | ZCash | Hold user's ZEC during the loan. Controlled by protocol. |
| User's viewing key | User's browser | Lets user see escrow balance and generate ZK proofs |
| OGBank contract | Avalanche | Verifies ZK proofs, manages loans, talks to Aave V3 |
| Proof verifier | Avalanche | Checks the math of the ZK proof (on-chain) |
| Aave V3 pool | Avalanche | Issues USDC borrows against protocol collateral |
| USDC | Avalanche | What the user receives after borrowing |
| Relayer | Off-chain | Manages ZCash escrow: creates addresses, returns ZEC |
| Proof generation | User's browser | Creates ZK proofs locally using the viewing key |

---

## Privacy Summary

| Data | Who Can See It |
|------|---------------|
| Your original ZCash address | Nobody on Avalanche (hidden by the escrow intermediary) |
| How much ZEC you deposited | You (via viewing key) and the protocol (via spending key). Not visible on Avalanche. |
| Your escrow address | The protocol knows it. Not linked to your identity on Avalanche. |
| Your USDC borrow amount | Public on Avalanche (required for Aave V3) |
| Your USDC repayment | Public on Avalanche |
| Connection between your ZCash and Avalanche identity | Only the protocol knows. Not visible on either chain publicly. |

---

## Failure Scenarios

| Scenario | What Happens | User Impact | Recovery |
|----------|-------------|-------------|----------|
| **Relayer is down** | Cannot create new accounts. Cannot return ZEC after repayment. | New users blocked. Existing borrowers can still repay on Avalanche but cannot claim ZEC until relayer recovers. | Relayer restart. ZEC is safe in escrow (spending key is derived, not lost). |
| **Proof generation fails in browser** | User cannot borrow or claim. | Blocked at borrow or claim step. | Retry. Check browser compatibility (WebAssembly required). Fallback: proof generation via CLI tool. |
| **ZCash node is down** | Relayer cannot create addresses or send ZEC. Viewing key scanning may fail. | Same as relayer down for new users. Existing deposits are unaffected. | ZCash node restart or failover to alternative node. |
| **Aave V3 is paused** | OGBankContract cannot borrow or repay. | Users with deposits cannot borrow. Users with loans cannot repay (and thus cannot claim ZEC). | Wait for Aave V3 to resume. ZEC remains safely in escrow. |
| **User loses viewing key** | Cannot generate proofs. Cannot verify deposit. Cannot claim ZEC. | Effectively locked out of their position. | Recovery via Kohaku wallet backup (if integrated). Or: relayer can re-derive viewing key if user proves identity via user_secret. |
| **ZEC price flash crash** | Positions become undercollateralized rapidly. | Liquidation triggered. Potential for bad debt if ZEC value drops below debt before liquidation completes. | Liquidation mechanism (see above). Over-collateralization buffer. |
| **Smart contract bug** | Proofs accepted when they shouldn't be, or rejected when they should be. | Potential fund loss (invalid proofs) or user lockout (valid proofs rejected). | Emergency pause function. Security audit before mainnet. Bug bounty program. |

---

## Deep Dive

For the full technical architecture with sequence diagrams, contract interfaces, and ZK circuit details:
- [Technical Architecture](../01_architecture.md)
- [Protocol Specification](../02_protocol.md)
- [Smart Contract Architecture](../03_contracts.md)
- [Privacy Model](../04_privacy-model.md)

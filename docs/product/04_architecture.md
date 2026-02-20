# Architecture — Product View

## How ZLend Works (3 Steps)

```
  1. LOCK              2. PROVE              3. BORROW
 ┌──────────┐      ┌──────────────┐      ┌──────────────┐
 │ Your ZCash│ ───▶ │ ZLend proves │ ───▶ │ You receive   │
 │ goes into │      │ you have     │      │ stablecoins   │
 │ a private │      │ enough       │      │ on Avalanche  │
 │ vault     │      │ collateral   │      │               │
 └──────────┘      └──────────────┘      └──────────────┘
                    (nobody sees
                     what you own)
```

**To get your ZCash back:** Repay the loan, prove you repaid, and your collateral is released.

---

## What You See vs. What Happens Behind the Scenes

### Locking Collateral

| What You See | What Happens Behind the Scenes |
|-------------|-------------------------------|
| "Enter ZEC amount" | Your browser creates a unique ZLend address using ZCash's key derivation (ZIP-32) |
| "Send ZEC to this address" | Your shielded ZEC goes to a ZCash address that only you control |
| "Collateral locked" | The smart contract on Avalanche records your collateral commitment — the *proof* that you locked it, not the amount or source |

### Borrowing

| What You See | What Happens Behind the Scenes |
|-------------|-------------------------------|
| "Choose borrow amount" | Your browser generates a mathematical proof that your collateral is sufficient — without revealing the collateral itself |
| Loading spinner | The proof is sent to Avalanche via a privacy relayer (not directly from your wallet). An on-chain verifier checks the math. |
| "Stablecoins received" | The smart contract tells Aave V3 to issue a borrow. Tokens arrive in your Avalanche wallet. |

### Repaying

| What You See | What Happens Behind the Scenes |
|-------------|-------------------------------|
| "Enter repay amount" | Standard ERC-20 token transfer to the ZLend contract |
| "Loan repaid" | ZLend forwards the repayment to Aave V3 |

### Withdrawing Collateral

| What You See | What Happens Behind the Scenes |
|-------------|-------------------------------|
| "Withdraw collateral" | Your browser generates a proof that the loan was fully repaid. This proof references a unique ID for this specific loan (preventing replay attacks). |
| "ZEC returned" | The contract verifies the proof, marks this loan as completed, and releases your shielded ZEC |

---

## The Trust Model

### "Your keys never leave your browser"

ZLend has two keys derived from your ZCash seed:

| Key | Who Has It | What It Does |
|-----|-----------|-------------|
| **Spending key** | Only you (in your browser) | Authorizes collateral release and withdrawals. Never transmitted anywhere. |
| **Viewing key** | You + ZLend contract + relayer | Verifies that your collateral exists. Cannot spend or move your funds. |

**Think of it like a safe deposit box:** The viewing key lets the bank confirm there's something inside. The spending key opens the box. Only you have the key to open it.

### The Relayer: A Sealed Courier

The ZLend relayer is a service that submits your transactions to Avalanche on your behalf.

**Why it exists:** If you submitted transactions directly from your wallet, someone could link your ZCash address to your Avalanche address. The relayer breaks that link.

**What the relayer CAN do:**
- See that *someone* is making a loan (not who)
- Delay or refuse to submit your transaction (you can switch relayers)

**What the relayer CANNOT do:**
- Steal your collateral (needs your spending key)
- Forge a proof (mathematically impossible)
- See how much collateral you have (only the viewing key, which proves existence, not amount)

**Analogy:** The relayer is like a courier carrying a sealed envelope. They deliver it, but they can't read what's inside or change the contents.

---

## What's On Which Network

```
┌───────────────────┐          ┌───────────────────────────────┐
│    ZCash Network   │          │    Avalanche C-Chain           │
│                    │          │                               │
│  Your shielded     │          │  ZLend smart contract         │
│  ZEC collateral    │ ──────▶ │  (orchestrates everything)    │
│                    │   proof  │         │                     │
│  (private, hidden) │          │         ▼                     │
│                    │          │  Aave V3 lending pool         │
│                    │          │  (issues the borrow)          │
│                    │          │         │                     │
│                    │          │         ▼                     │
│                    │          │  Stablecoins → your wallet    │
└───────────────────┘          └───────────────────────────────┘
```

| Component | Network | What It Does |
|-----------|---------|-------------|
| Your ZEC collateral | ZCash | Stays in a shielded address you control |
| ZLend contract | Avalanche | Verifies proofs, manages loans, talks to Aave |
| Proof verifier | Avalanche | Checks the math of your ZK proof (on-chain) |
| Aave V3 pool | Avalanche | Actually issues and manages the borrow |
| Stablecoins | Avalanche | What you receive after borrowing |
| Relayer | Off-chain | Submits transactions privately |
| Proof generation | Your browser | Creates the ZK proof locally |

---

## Privacy Summary

| Data | Who Can See It |
|------|---------------|
| Your ZCash address | Nobody (hidden by ZK proof) |
| How much ZEC you have | Nobody (proof says "enough" without revealing amount) |
| Your spending key | Only you (never leaves your browser) |
| That you made a loan | The relayer knows someone did, but not who |
| Your borrow amount | Public on Avalanche (required for Aave V3) — *Phase 2 will encrypt this* |
| Your repayment | Public on Avalanche — *Phase 2 will encrypt this* |
| Connection between your ZCash and Avalanche identity | Nobody (relayer breaks the link) |

---

## Deep Dive

For the full technical architecture with sequence diagrams, contract interfaces, and ZK circuit details:
- [Technical Architecture](../01_architecture.md)
- [Protocol Specification](../02_protocol.md)
- [Smart Contract Architecture](../03_contracts.md)
- [Privacy Model](../04_privacy-model.md)

# OGBank System Architecture

OGBank is a privacy-preserving lending protocol on **Avalanche** that uses **ZCash** shielded UTXOs as collateral to borrow ERC-20 tokens via **Aave V3**. Zero-knowledge proofs (Ultrahonk) verify collateral ownership without revealing the user's ZCash address or balance on-chain.

![Architecture Overview](../assets/architecture-overview.png)

---

## High-Level Overview

```
┌──────────────┐     ┌──────────────┐     ┌─────────────────────────────────────┐
│   Browser    │────▶│  ZCash Node  │────▶│        Avalanche (C-Chain)          │
│              │     │              │     │                                     │
│ - Adapter    │     │ createOGBank  │     │ ┌─────────────┐  ┌──────────────┐  │
│ - Balance    │     │  Address()   │     │ │ OGBankContract│  │ Ultrahonk    │  │
│ - Relayer    │     │              │     │ │              │──│ Verifier     │  │
│ - Privacy    │     │  ──▶ d       │     │ │              │  └──────────────┘  │
│   Pools      │     │              │     │ │              │                    │
│              │     │  Spending Key│     │ │              │  ┌──────────────┐  │
│              │     │  & OGBank     │     │ │              │──│ Aave V3      │  │
│              │     │  Relayer     │     │ │              │  │ (Pool)       │  │
│              │     │              │     │ └─────────────┘  └──────────────┘  │
│              │     │ OGBank =      │     │                                     │
│              │     │ H(X, ZIP32)  │     │ ┌─────────────┐                    │
│              │     │  ──▶ vk, sk  │     │ │ ProtoSocolo  │                    │
│              │     │              │     │ │ (ERC-20)     │                    │
│              │     │              │     │ └─────────────┘                    │
└──────────────┘     └──────────────┘     └─────────────────────────────────────┘
```

---

## Components

### Browser Client

The user-facing application that coordinates the full lending cycle:

| Module | Responsibility |
|--------|---------------|
| **Adapter** | Wallet connection and transaction signing for both ZCash and Avalanche |
| **Balance** | Tracks ZCash UTXO balances and Avalanche ERC-20 positions |
| **Relayer** | Submits privacy-preserving transactions to Avalanche on behalf of the user |
| **Privacy Pools** | Manages shielded pool interactions and proof generation |

### ZCash Node

Provides the collateral layer:

- **`createOGBankAddress()`** — Generates a deterministic OGBank address `d` using ZCash's ZIP-32 hierarchical deterministic key derivation
- **UTXO Management** — Supplies unspent transaction outputs as collateral proof inputs
- **Key Derivation** — Produces the viewing key (`vk`) and spending key (`sk`) pair used throughout the protocol

### Spending Key & OGBank Relayer

The core cryptographic bridge between ZCash and Avalanche:

```
OGBank = H(X, ZIP32) → vk, sk
```

Where:
- `H` is a hash function binding the user's identity `X` to the ZIP-32 derivation path
- `vk` (viewing key) — allows the protocol to verify collateral without spending it
- `sk` (spending key) — retained by the user, never leaves the client

The relayer holds derived keys to submit transactions on Avalanche without revealing the user's ZCash origin address.

### Avalanche Contracts

| Contract | Role |
|----------|------|
| **OGBankContract** | Core orchestrator — handles supply, borrow, repay, and withdraw flows |
| **ProtoSocolo (ERC-20)** | Token contract for internal ERC-20 transfers within the protocol |
| **Ultrahonk Verifier** | On-chain ZK proof verifier (Noir/Barretenberg) that validates collateral proofs |
| **Aave V3 (Pool)** | External lending pool — receives collateral supply and issues borrows |

---

## Data Flow

```mermaid
sequenceDiagram
    participant U as User (Browser)
    participant Z as ZCash Node
    participant R as OGBank Relayer
    participant C as OGBankContract
    participant V as Ultrahonk Verifier
    participant A as Aave V3

    U->>Z: createOGBankAddress()
    Z-->>U: deterministic address (d)

    U->>R: requestUTXOs
    R-->>U: nonce, vk, nullifier

    U->>C: SupplyTransfer(amount, UTk)
    U->>C: connectVk

    Note over U: generateProof(borrow)
    U->>C: Borrow(proof, amount)
    C->>V: verify(proof)
    V-->>C: Success
    C->>A: approval(addr, amount)
    C->>A: supply(amount)
    C->>A: borrow(amount)
    C-->>U: ERC20Transfer via ProtoSocolo

    U->>C: Repay(amount)

    Note over U: generateProof(repaying)
    U->>C: withdrawProof(amount)
    C->>V: verify(proof)
    V-->>C: Success
    C-->>U: logs: FinishPayment(ogbank, amount, recipient, address)
```

---

## Network Topology

| Layer | Network | Purpose |
|-------|---------|---------|
| Collateral | ZCash (mainnet) | Shielded UTXOs as collateral source |
| Execution | Avalanche C-Chain | Smart contracts, Aave V3 integration |
| Proving | Off-chain (client) | Noir circuits compiled to Ultrahonk proofs |
| Relaying | OGBank Relayer | Privacy-preserving transaction submission |

---

## Phased Privacy Roadmap

Privacy features are delivered incrementally across three phases:

### Phase 1 — MVP

| Feature | Implementation |
|---------|---------------|
| Collateral privacy | ZK proofs (Ultrahonk/Noir) verify UTXO ownership without revealing ZCash address |
| Replay protection | Nullifier-per-borrow-cycle + state root binding ([Protocol Spec](02_protocol.md#nullifier-per-borrow-cycle)) |
| Token transfers | Standard ERC-20 (ProtoSocolo) — borrow amounts are public |
| Event matching | Hash-based filtering: `H(vk, event_data)` — client-side matching |
| Key derivation | ZIP-32 Sapling: `m_Sapling / 32' / 133' / account' / ogbank_index` ([Details](05_zcash-integration.md#zip-32-derivation-details)) |

### Phase 2 — Enhanced Privacy

| Feature | Implementation |
|---------|---------------|
| Private balances | ZAMA fhEVM encrypted ERC-20 (once Avalanche support ships, expected H1 2026) |
| Private event matching | ZK regex via [noir-zk-regex](https://github.com/hashcloak/noir-zk-regex) (once Noir target matures) |
| Privacy Pools | Adapt [Kohaku](https://github.com/ethereum/kohaku) `@kohaku-eth/privacy-pools` for deposit set membership and exclusion proofs |

### Phase 3 — Advanced Features

| Feature | Implementation |
|---------|---------------|
| Threshold signatures | BLS threshold schemes via Shamir's Secret Sharing for multi-sig collateral |
| Railgun integration | Shielded ERC-20 transfers following Kohaku `@kohaku-eth/railgun` patterns |
| ZK compliance | Zero-knowledge AML/FT attestations — prove compliance without revealing identity |

---

## Inspiration

The pool supply/borrow pattern is modeled after [SparkLend Core Contracts — Pool#supply](https://docs.spark.fi/dev/sparklend/core-contracts/pool#supply), adapted to work with ZK-verified cross-chain collateral from ZCash.

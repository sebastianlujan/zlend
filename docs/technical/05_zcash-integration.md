# ZCash Integration

## Overview

ZLend uses ZCash's shielded transaction model as the collateral layer. The protocol interfaces with ZCash via JSON-RPC to validate addresses, fetch UTXOs, and verify transaction proofs. Key derivation follows the ZIP-32 standard for hierarchical deterministic wallets (Sapling/Orchard).

![ZCash Interfaces Research](../assets/zcash-interfaces-research.png)

---

## ZCash JSON-RPC Interface

The following RPC methods are used or available for ZLend's ZCash integration:

### Address & Validation

| Method | Purpose |
|--------|---------|
| `validateaddress` | Validate a ZCash address format and type (transparent/shielded) |

### Block Data

| Method | Purpose |
|--------|---------|
| `getblockchaininfo` | Current chain state, height, difficulty |
| `getblock` | Full block data by hash |
| `getblockcount` | Current block height |
| `getblockstats` | Per-block statistics |
| `getblockheader` | Block header by hash |
| `getblockhash` | Block hash by height |
| `getbestblockhash` | Hash of the current tip |
| `getdifficulty` | Current mining difficulty |
| `getchaintips` | All known chain tips |

### Transaction Management

| Method | Purpose |
|--------|---------|
| `sendrawtransaction` | Broadcast a signed transaction |
| `createrawtransaction` | Build an unsigned transaction |
| `getrawtransaction` | Fetch raw transaction data by txid |
| `gettxout` | Get specific UTXO by txid and vout index |
| `decoderawtransaction` | Decode a raw transaction hex |
| `decodescript` | Decode a script hex |

### Mempool

| Method | Purpose |
|--------|---------|
| `getmempoolinfo` | Mempool size and state |
| `getrawmempool` | All transaction IDs in mempool |
| `getmempoolancestors` | Ancestor transactions of a mempool entry |
| `getmempooldescendants` | Descendant transactions of a mempool entry |
| `getmempoolentry` | Detailed mempool data for a specific txid |

### Proof & Verification

| Method | Purpose |
|--------|---------|
| `verifymessage` | Verify a signed message |
| `verifytxoutproof` | Verify a UTXO inclusion proof |
| `gettxoutproof` | Generate a UTXO inclusion proof (Merkle proof) |

### Fee Estimation

| Method | Purpose |
|--------|---------|
| `estimatesmartfee` | Estimate fee rate for confirmation target |

---

## Key Libraries & Tools

### ZCash Primitives (JavaScript)

**Repository**: [zcash-hackworks/zcash-primitives-js](https://github.com/zcash-hackworks/zcash-primitives-js/tree/master/src)

JavaScript implementation of ZCash cryptographic primitives. Used for:
- Key derivation (ZIP-32 Sapling/Orchard paths)
- Address generation (`createZLendAddress()`)
- Note encryption/decryption
- Viewing key operations

### ZCash RPC Documentation

**Reference**: [zcash-rpc.github.io](https://zcash-rpc.github.io/)

Official ZCash RPC method documentation and parameter specifications.

### UTXO Fetching (Tatum)

- **UTXO API**: [Postman — blockchainnow/tatum-io — get-utxos-200](https://www.postman.com/blockchainnow/tatum-io/request/kqirish/get-utxos-200)
- **Address Validation**: [docs.tatum.io — rpc-zcash-validateaddress](https://docs.tatum.io/reference/rpc-zcash-validateaddress)

Tatum provides REST APIs for fetching ZCash UTXOs and validating addresses without running a full node.

---

## ZK Tooling

### Noir ZK Regex

**Repository**: [hashcloak/noir-zk-regex](https://github.com/hashcloak/noir-zk-regex)

Noir library for zero-knowledge regular expression matching. Used for:
- Matching viewing key patterns against on-chain event data
- Pattern: `regexp(p, event)` — proves a viewing key matches an event without revealing the key

### zk-regex-v2

**Reference**: [zk.email/blog/zk-regex-v2](https://zk.email/blog/zk-regex-v2)

Second generation ZK regex implementation from zk.email. Provides the theoretical foundation for pattern matching within ZK circuits.

---

## Viewing Key Generation

ZCash viewing keys (`vk`) are derived through the ZIP-32 key tree:

```
Master Seed
    └── ZIP-32 Derivation Path
          └── Spending Key (sk) ── kept client-side
          └── Viewing Key (vk) ── shared with protocol
                └── Incoming Viewing Key (ivk)
                └── Full Viewing Key (fvk)
                └── Outgoing Viewing Key (ovk)
```

### Usage in ZLend

- **`vk`** is shared with the ZLend Relayer and ZLendContract to verify UTXO ownership
- **`sk`** is never transmitted — stays in the user's browser client
- **Event matching**: Hash-based filtering for MVP (`H(vk, event_data)`), ZK regex for Phase 2
- **Deterministic derivation**: `ZLend = H(X, ZIP32) → vk, sk` ensures the same user always derives the same key pair

---

## ZIP-32 Derivation Details

Full key derivation from master seed, per [ZIP-32 Sapling specification](https://zips.z.cash/zip-0032):

### Step 1 — Master Key Generation

Starting from a seed `S` (32-252 bytes):

```
I = BLAKE2b-512("ZcashIP32Sapling", S)
I_L = left 32 bytes  → master spending key (sk)
I_R = right 32 bytes → master chain code (c)
```

### Step 2 — Expanded Key Components

Using `PRF^expand` (BLAKE2b-512 with Zcash-specific personalization):

| Key Component | Derivation | Type |
|---------------|-----------|------|
| `ask` (spend authorizing key) | `PRF^expand(sk, 0x00) mod r_J` | Jubjub scalar |
| `nsk` (nullifier private key) | `PRF^expand(sk, 0x01) mod r_J` | Jubjub scalar |
| `ovk` (outgoing viewing key) | `PRF^expand(sk, 0x02)[0..32]` | 32 bytes |
| `dk` (diversifier key) | `PRF^expand(sk, 0x10)[0..32]` | 32 bytes |

### Step 3 — Public Key Derivation

| Public Key | Derivation | Description |
|-----------|-----------|-------------|
| `ak` (spend validating key) | `ask * G_spend` | Jubjub curve point |
| `nk` (nullifier deriving key) | `nsk * G_proof` | Jubjub curve point |
| `ivk` (incoming viewing key) | `CRH^ivk(ak, nk)` | Scalar — enables scanning for incoming notes |

### Step 4 — Composite Keys

| Key Set | Components | Usage |
|---------|-----------|-------|
| **Extended Spending Key** | `(ask, nsk, ovk, dk, c)` | Full spending authority + hierarchical derivation |
| **Full Viewing Key (fvk)** | `(ak, nk, ovk, dk)` | Verify transactions, derive child viewing keys, cannot spend |
| **Incoming Viewing Key (ivk)** | Derived from `(ak, nk)` | Scan for incoming notes/events only |

### Derivation Path

BIP-44 adapted for ZCash Sapling:

```
m_Sapling / purpose' / coin_type' / account'
```

| Level | Value | Meaning |
|-------|-------|---------|
| `purpose` | `32'` | ZIP-32 standard |
| `coin_type` | `133'` | ZEC (per SLIP-44) |
| `account` | `0'`, `1'`, ... | Wallet account divisions |

**For ZLend-specific addresses**, use a deeper path:

```
m_Sapling / 32' / 133' / account' / zlend_index
```

This avoids collision with standard ZCash wallet addresses.

### Diversifier Mechanism

The diversifier key `dk` generates up to 2^88 distinct payment addresses via **FF1-AES256** encryption. Approximately 50% of diversifiers produce valid Jubjub points, yielding ~2^87 usable addresses per key set.

### Cryptographic Primitives

| Primitive | Usage |
|-----------|-------|
| `BLAKE2b-512` | Key material expansion (personalized with "ZcashIP32Sapling") |
| `PRF^expand` | BLAKE2b-512 with domain separation for key derivation |
| Jubjub curve | Elliptic curve arithmetic (order `r_J`) |
| `FF1-AES256` | Format-preserving encryption for diversifier generation |
| `CRH^ivk` | Collision-resistant hash for incoming viewing key |

### Mapping to ZLend

| ZLend Concept | ZIP-32 Component |
|---------------|------------------|
| `sk` (spending key) | Extended spending key `(ask, nsk, ovk, dk, c)` |
| `vk` (viewing key) | Full viewing key `(ak, nk, ovk, dk)` |
| `ivk` | Incoming viewing key — for scanning Avalanche events |
| `d` (deterministic address) | Diversified payment address derived from `ivk * G_d` |

---

## Integration Architecture

```mermaid
graph LR
    subgraph "ZCash Network"
        ZN[ZCash Node]
        UTXO[(Shielded UTXOs)]
    end

    subgraph "Off-chain"
        BRW[Browser Client]
        REL[ZLend Relayer]
        TAT[Tatum API]
    end

    subgraph "Avalanche"
        ZLC[ZLendContract]
    end

    BRW -->|createZLendAddress| ZN
    ZN -->|deterministic addr d| BRW
    BRW -->|requestUTXOs| REL
    REL -->|nonce, vk, nullifier| BRW
    TAT -->|UTXO data| REL
    BRW -->|SupplyTransfer, connectVk| ZLC
    ZN --- UTXO
```

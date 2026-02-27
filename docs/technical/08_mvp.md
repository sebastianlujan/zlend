# OGBank MVP — Architecture & Diagrams

This document describes the MVP Proof-of-Concept implementation located in `MVP-POC/`.

## System Architecture

```mermaid
graph TB
    subgraph User ["User"]
        CLI["ogbank-cli<br/>(Rust binary)"]
        WALLET["Any Zcash Wallet<br/>(zecwallet, ywallet)"]
    end

    subgraph Relayer ["OGBank Relayer (localhost:3000)"]
        API["axum REST API"]
        CRYPTO["SK Encryption<br/>(ChaCha20-Poly1305)"]
        SCAN["Trial Decryption<br/>Engine (ogbank-core)"]
        EVM["EVM Signer<br/>(alloy)"]
        DB[(SQLite<br/>positions + notes<br/>+ loans<br/>sk encrypted)]
    end

    subgraph Zcash ["Zcash Network (testnet)"]
        CHAIN["Shielded Pool<br/>Commitment Tree"]
    end

    subgraph External ["External Provider"]
        TATUM["Tatum REST API<br/>api.tatum.io/v3/zcash"]
    end

    subgraph Avalanche ["Avalanche C-Chain (Fuji testnet)"]
        CONTRACT["OGBankMVP.sol<br/>(trusts relayer attestation)"]
        TOKEN["MockUSDC.sol<br/>(ERC-20 test token)"]
    end

    CLI -- "1. generate<br/>H(X,ZIP32) → sk,vk,d" --> CLI
    CLI -- "2. register<br/>POST /register {sk,vk,d}" --> API
    WALLET -- "3. deposit ZEC<br/>shielded tx → address d" --> CHAIN
    CLI -- "4. scan --txid<br/>POST /scan/:id {txid}" --> API
    API -- "5. fetch tx" --> TATUM
    TATUM -- "6. full tx data<br/>+ encrypted outputs" --> SCAN
    SCAN -- "7. try_decrypt(ivk, outputs)<br/>→ value, memo" --> DB
    CLI -- "8. borrow<br/>POST /borrow/:id {amount, recipient}" --> API
    EVM -- "9. sign attestation +<br/>submit borrow tx" --> CONTRACT
    CONTRACT -- "10. transfer<br/>ERC-20" --> TOKEN
    TOKEN -- "11. tokens to<br/>borrower address" --> CLI
    CLI -- "12. balance<br/>GET /balance/:id" --> API

    style CLI fill:#2d5a27,color:#fff
    style API fill:#1a3a5c,color:#fff
    style SCAN fill:#1a3a5c,color:#fff
    style EVM fill:#1a3a5c,color:#fff
    style CRYPTO fill:#8b0000,color:#fff
    style DB fill:#4a3728,color:#fff
    style CHAIN fill:#5c3a1a,color:#fff
    style TATUM fill:#3a1a5c,color:#fff
    style WALLET fill:#2d5a27,color:#fff
    style CONTRACT fill:#8b4513,color:#fff
    style TOKEN fill:#8b4513,color:#fff
```

## Full Sequence: Generate → Stake → Verify → Borrow

```mermaid
sequenceDiagram
    actor User
    participant CLI as ogbank-cli
    participant Relayer as ogbank-relayer<br/>(localhost:3000)
    participant Tatum as Tatum API
    participant Zcash as Zcash Testnet

    Note over User,Zcash: Phase 0: Identity Creation (offline)
    User->>CLI: ogbank-cli generate
    CLI->>CLI: seed = bip39::Mnemonic::generate(24 words)
    CLI->>CLI: H(seed, ZIP32) → sk, fvk, ivk, ovk, address(d)
    CLI-->>User: mnemonic + address d + keys
    CLI->>CLI: save ~/.ogbank/keys.json

    Note over User,Zcash: Phase 1: Register with Relayer
    User->>CLI: ogbank-cli register --relayer http://localhost:3000
    CLI->>Relayer: POST /register { sk, vk, ivk, address }
    Relayer->>Relayer: encrypt_sk(sk, RELAYER_SK_PASSPHRASE)
    Relayer->>Relayer: INSERT INTO positions(id, sk_encrypted, vk, ivk, address)
    Relayer-->>CLI: { id: "uuid-1234", address: "zs1..." }
    CLI-->>User: Registered! ID: uuid-1234

    Note over User,Zcash: Phase 2: Deposit ZEC (external wallet)
    User->>Zcash: Send 1.5 ZEC to address d (shielded tx)
    Zcash->>Zcash: Note committed to tree, encrypted with ivk
    Zcash-->>User: txid: "abc123..."

    Note over User,Zcash: Phase 3: Verify Deposit
    User->>CLI: ogbank-cli scan --relayer ... --txid abc123
    CLI->>Relayer: POST /scan/uuid-1234 { txid: "abc123" }
    Relayer->>Tatum: GET /v3/zcash/transaction/abc123
    Tatum-->>Relayer: { orchard_actions: [...encrypted outputs...] }
    Relayer->>Relayer: for each action: try_decrypt(ivk, ciphertext)
    alt Decryption succeeds
        Relayer->>Relayer: INSERT INTO notes(position_id, txid, value_zat)
        Relayer->>Relayer: UPDATE positions SET balance_zat += value
        Relayer-->>CLI: { found: true, value_zat: 150000000 }
    else Decryption fails
        Relayer-->>CLI: { found: false }
    end
    CLI-->>User: Deposit found! 1.5 ZEC staked

    Note over User,Zcash: Phase 4: Borrow ERC-20 (Relayer Attestation)
    User->>CLI: ogbank-cli borrow --relayer ... --amount 100 --recipient 0xAlice
    CLI->>Relayer: POST /borrow/uuid-1234 { amount: 100e6, recipient: "0xAlice" }
    Relayer->>Relayer: CHECK: balance_zat >= amount * 150% (collateral ratio)

    participant Avalanche as OGBankMVP.sol<br/>(Fuji testnet)

    Relayer->>Avalanche: borrow(borrower, collateralZat, amount, positionId)
    Avalanche->>Avalanche: require(msg.sender == trustedRelayer)
    Avalanche->>Avalanche: MockUSDC.transfer(borrower, amount)
    Avalanche-->>Relayer: tx receipt (success)
    Relayer->>Relayer: INSERT INTO loans(position_id, amount, tx_hash)
    Relayer-->>CLI: { success: true, tx_hash: "0xabc...", borrowed: 100e6 }
    CLI-->>User: Borrowed 100 USDC! tx: 0xabc...

    Note over User,Zcash: Phase 5: Check Balance
    User->>CLI: ogbank-cli balance --relayer ...
    CLI->>Relayer: GET /balance/uuid-1234
    Relayer-->>CLI: { balance_zat: 150000000, borrowed_zat: 100000000, notes: 1 }
    CLI-->>User: Staked: 1.5 ZEC (1 note) | Borrowed: 100 USDC
```

## Key Derivation Tree

```mermaid
graph TD
    SEED["seed (32 bytes)<br/>from BIP-39 mnemonic"]
    SK["sk (spending key)<br/>ZIP-32 derivation path:<br/>m/32'/133'/0'"]
    ASK["ask<br/>(spend authorizing key)<br/>Pallas scalar"]
    NK["nk<br/>(nullifier key)<br/>Pallas base"]
    RIVK["rivk<br/>(raw incoming viewing key)<br/>Pallas scalar"]
    AK["ak = [ask] x G_Orchard<br/>(public spend verification)"]
    FVK["fvk = (ak, nk, rivk)<br/>(full viewing key)"]
    IVK["ivk<br/>(incoming viewing key)<br/>for trial decryption"]
    OVK["ovk<br/>(outgoing viewing key)"]
    DK["dk<br/>(diversifier key)"]
    ADDR["address d<br/>(OGBank shielded address)"]

    SEED --> SK
    SK -->|"ToScalar(PRF(sk,[0x06]))"| ASK
    SK -->|"ToBase(PRF(sk,[0x07]))"| NK
    SK -->|"ToScalar(PRF(sk,[0x08]))"| RIVK
    ASK --> AK
    AK --> FVK
    NK --> FVK
    RIVK --> FVK
    FVK -->|"Commit_ivk(rivk,ak,nk)"| IVK
    FVK --> OVK
    FVK --> DK
    DK --> ADDR

    style SEED fill:#5c3a1a,color:#fff
    style SK fill:#8b0000,color:#fff
    style ASK fill:#8b0000,color:#fff
    style FVK fill:#1a3a5c,color:#fff
    style IVK fill:#2d5a27,color:#fff
    style ADDR fill:#2d5a27,color:#fff

    classDef secret fill:#8b0000,color:#fff,stroke:#ff0000
    classDef shared fill:#1a3a5c,color:#fff
    classDef public fill:#2d5a27,color:#fff

    class SK,ASK,NK,RIVK secret
    class FVK,OVK,DK shared
    class IVK,ADDR,AK public
```

## Data Model (SQLite)

```mermaid
erDiagram
    POSITIONS {
        text id PK "uuid"
        text address "zs1... (OGBank address)"
        blob vk "full viewing key"
        blob sk_encrypted "encrypted spending key (ChaCha20-Poly1305)"
        blob ivk "incoming viewing key"
        int balance_zat "total staked (zatoshis)"
        int borrowed_zat "total borrowed (token units)"
        int note_count "number of unspent notes"
        text created_at "timestamp"
        text updated_at "timestamp"
    }

    NOTES {
        int id PK "autoincrement"
        text position_id FK "references positions.id"
        text txid "zcash transaction hash"
        int value_zat "note value in zatoshis"
        blob nullifier "note nullifier"
        blob memo "decrypted memo (512 bytes)"
        text found_at "timestamp"
        int spent "0=unspent, 1=spent"
    }

    LOANS {
        int id PK "autoincrement"
        text position_id FK "references positions.id"
        int amount "borrowed amount (token units)"
        text tx_hash "Avalanche transaction hash"
        text recipient "EVM address that received tokens"
        text status "active / repaid"
        text created_at "timestamp"
    }

    POSITIONS ||--o{ NOTES : "has many"
    POSITIONS ||--o{ LOANS : "has many"
```

## Crate Dependency Graph

```mermaid
graph LR
    subgraph Workspace ["Cargo Workspace"]
        CORE["ogbank-core<br/>(library)"]
        CLI["ogbank-cli<br/>(binary)"]
        RELAY["ogbank-relayer<br/>(binary)"]
    end

    subgraph Zcash ["Zcash Crates"]
        ZK["zcash_keys"]
        ZP["zcash_primitives"]
        ZPR["zcash_protocol"]
        ORD["orchard"]
        ZNE["zcash_note_encryption"]
    end

    subgraph Infra ["Infrastructure"]
        AXUM["axum"]
        CLAP["clap"]
        REQ["reqwest"]
        SQL["rusqlite"]
        TOK["tokio"]
        BIP["bip39"]
        ALLOY["alloy<br/>(EVM signing + tx)"]
    end

    subgraph Solidity ["Foundry (contracts/)"]
        OGBANK_SOL["OGBankMVP.sol"]
        MOCK["MockUSDC.sol"]
    end

    CLI --> CORE
    CLI --> CLAP
    CLI --> REQ
    CLI --> TOK

    RELAY --> CORE
    RELAY --> AXUM
    RELAY --> REQ
    RELAY --> SQL
    RELAY --> TOK
    RELAY --> ALLOY

    CORE --> ZK
    CORE --> ZP
    CORE --> ZPR
    CORE --> ORD
    CORE --> ZNE
    CORE --> BIP

    RELAY -.->|"calls"| OGBANK_SOL
    OGBANK_SOL -->|"transfers"| MOCK

    style CORE fill:#2d5a27,color:#fff
    style CLI fill:#1a3a5c,color:#fff
    style RELAY fill:#5c3a1a,color:#fff
    style OGBANK_SOL fill:#8b4513,color:#fff
    style MOCK fill:#8b4513,color:#fff
```

## MVP vs Production Scope

```mermaid
graph TB
    subgraph MVP ["MVP (this implementation)"]
        M1["ZIP-32 key derivation"]
        M2["sk encrypted at rest (ChaCha20-Poly1305 + Argon2)"]
        M3["Deposit verification via trial decryption"]
        M4["Balance tracking (SQLite)"]
        M5["Tatum API (Plan B)"]
        M6["CLI tool"]
        M7["Relayer attestation borrow"]
        M8["OGBankMVP.sol on Avalanche Fuji"]
        M9["MockUSDC ERC-20 transfer"]
    end

    subgraph V1 ["v1 (next)"]
        V1A["FROST 2-of-3 replaces encrypted sk"]
        V1B["Upgrade to full OGBankContract"]
        V1C["Unified 2-mode Noir circuit + Ultrahonk verifier"]
        V1D["Full chain scanning"]
        V1E["Repay + withdraw via auth mode (nullifier chain)"]
        V1F["Aave V3 integration"]
        V1G["Debt Notes — UTXO-style debt Merkle tree"]
        V1H["Partial repayments via change notes"]
        V1I["Chainlink ZEC/USD price oracle"]
    end

    subgraph V2 ["v2 (future)"]
        V2A["FHE-encrypted operations (Fhenix)"]
        V2B["Privacy Pools"]
        V2C["Liquidation mechanism"]
        V2D["Chainlink CCIP"]
        V2E["Fhenix cUSDC (encrypted balances)"]
        V2F["Halo2 recursive proofs (native Orchard verification)"]
    end

    MVP --> V1 --> V2

    style MVP fill:#2d5a27,color:#fff
    style V1 fill:#1a3a5c,color:#fff
    style V2 fill:#5c3a1a,color:#fff
```

## SK Security — Encryption at Rest

The spending key (`sk`) is encrypted before storage using **ChaCha20-Poly1305** with a key derived from a passphrase via **Argon2id**.

```
Registration:
  CLI sends sk → Relayer encrypts sk with ChaCha20-Poly1305 → stores ciphertext in SQLite

Operations needing sk:
  Relayer reads ciphertext from SQLite → decrypts with in-memory key → uses sk → zeroes memory

Encryption key derivation:
  RELAYER_SK_PASSPHRASE (env var) → Argon2id(passphrase, salt) → 256-bit symmetric key
  Key lives only in memory. Never written to disk.

Storage format:
  salt(16 bytes) || nonce(12 bytes) || ciphertext
```

### Threat Model

| Threat | Protected? |
|--------|:----------:|
| SQLite file stolen/leaked | Yes — sk is ciphertext |
| Database backup exfiltrated | Yes — useless without passphrase |
| Relayer process memory dump | No (sk decrypted in memory during use) |
| Relayer operator (has passphrase) | No (custodial trust model for MVP) |

### Upgrade Path to FROST

1. Replace `encrypt_sk` / `decrypt_sk` with FROST key splitting
2. Relayer stores only 1 FROST share (not encrypted full sk)
3. User holds 2 shares (2-of-3 threshold met without relayer)
4. Uses `frost-rerandomized` crate from `ZcashFoundation/frost`
5. Full `ask` is **never reconstructed** — partial signatures combine into valid RedPallas

## Directory Structure

```
MVP-POC/
├── Cargo.toml              (workspace root)
├── .gitignore
├── crates/
│   ├── ogbank-core/         (shared Zcash crypto)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── keys.rs     (ZIP-32 derivation)
│   │       ├── scan.rs     (trial decryption)
│   │       └── crypto.rs   (ChaCha20-Poly1305 encryption)
│   │
│   ├── ogbank-cli/          (CLI binary)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs     (generate, register, balance, scan, borrow)
│   │
│   └── ogbank-relayer/      (relayer binary)
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs     (axum server startup)
│           ├── api.rs      (REST endpoints)
│           ├── db.rs       (SQLite schema + queries)
│           ├── zcash.rs    (Tatum API client)
│           └── evm.rs      (alloy EVM signer + contract calls)
│
└── contracts/              (Foundry project)
    ├── foundry.toml
    ├── src/
    │   ├── OGBankMVP.sol    (core contract — relayer attestation borrow)
    │   └── MockUSDC.sol    (test ERC-20 token)
    ├── script/
    │   └── Deploy.s.sol    (deployment script for Fuji testnet)
    └── test/
        └── OGBankMVP.t.sol  (contract tests)
```

## Quick Start

```bash
# Build Rust workspace
cd MVP-POC && cargo build

# Run tests
cargo test

# Run Solidity tests
cd contracts && forge test

# Generate identity
cargo run -p ogbank-cli -- generate

# Start relayer
RELAYER_SK_PASSPHRASE=my-secret cargo run -p ogbank-relayer

# Register with relayer
cargo run -p ogbank-cli -- register --relayer http://localhost:3000

# Check balance
cargo run -p ogbank-cli -- balance --relayer http://localhost:3000
```

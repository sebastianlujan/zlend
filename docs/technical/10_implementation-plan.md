# Plan: ZCash Core Implementation — ogbank-core + ogbank-cli + ogbank-relayer

## Context

OGBank tiene una especificación técnica completa (9 documentos, ~15K líneas) pero **cero implementación**. Este plan establece la ruta incremental y TDD para construir los 3 crates Rust del MVP descritos en [08_mvp.md](docs/technical/08_mvp.md), basándose en la arquitectura documentada.

**Objetivo**: Implementar el core criptográfico de ZCash (ZIP-32, trial decryption, SK encryption), el CLI de usuario, y el relayer — todo con TDD estricto, resolviendo incrementalmente cada componente.

**Entregable final**: Un documento `docs/technical/10_implementation-plan.md` con el plan completo + una skill `zlend` actualizada que codifica los patrones de código para mantener uniformidad.

---

## Pre-requisitos identificados en docs

De la lectura exhaustiva de `docs/`:

| Dependencia | Crate Rust | Versión | Uso |
|-------------|-----------|---------|-----|
| ZIP-32 key derivation | `zcash_keys` | latest | `keys.rs` — seed → sk → fvk → ivk → d |
| Orchard protocol | `orchard` | latest | `scan.rs` — trial decryption, note structure |
| Zcash primitives | `zcash_primitives` | latest | Protocol types, tx building |
| Zcash protocol | `zcash_protocol` | latest | Network, consensus params |
| Note encryption | `zcash_note_encryption` | latest | ChaCha20-Poly1305 for note decrypt |
| BIP-39 mnemonics | `bip39` | latest | Seed generation |
| HTTP client | `reqwest` | latest | Tatum API, Zcash RPC |
| Async runtime | `tokio` | latest | Relayer server, background tasks |
| Web framework | `axum` | latest | REST API |
| Database | `rusqlite` | latest | SQLite persistence |
| EVM signing | `alloy` | latest | Avalanche C-Chain interaction |
| CLI | `clap` | latest | Command parsing |
| SK encryption | `chacha20poly1305` + `argon2` | latest | `crypto.rs` |

---

## Fase 0 — Scaffolding y Skill

### 0.1 Crear workspace Cargo

```
MVP-POC/
├── Cargo.toml              (workspace root)
├── crates/
│   ├── ogbank-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── keys.rs
│   │       ├── scan.rs
│   │       └── crypto.rs
│   ├── ogbank-cli/
│   │   ├── Cargo.toml
│   │   └── src/main.rs
│   └── ogbank-relayer/
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── api.rs
│           ├── db.rs
│           ├── zcash.rs
│           └── evm.rs
└── contracts/
    ├── foundry.toml
    └── src/
        ├── OGBankMVP.sol
        └── MockUSDC.sol
```

### 0.2 Crear/actualizar skill `zlend`

La skill debe codificar:
- Patrones de error handling (thiserror + anyhow)
- Convenciones de testing (unit tests inline, integration tests en `tests/`)
- Patrones de serialización (serde para JSON, hex encoding para bytes)
- Convenciones de logging (tracing crate)
- Patrones de configuración (env vars + dotenv)
- Naming conventions del protocolo (borrow_nullifier, repay_nullifier, etc.)

### 0.3 TDD Ground Rules

Cada componente sigue este ciclo:
1. **Red**: Escribir test que falla
2. **Green**: Implementar lo mínimo para pasar el test
3. **Refactor**: Limpiar sin romper tests
4. Cada fase tiene un `cargo test` al final que debe pasar al 100%

---

## Fase 1 — ogbank-core/keys.rs — ZIP-32 Key Derivation

**Referencia**: [05_zcash-integration.md §ZIP-32](docs/technical/05_zcash-integration.md), [08_mvp.md §Key Derivation Tree](docs/technical/08_mvp.md)

### Qué implementar

El core criptográfico de OGBank: derivar todo el árbol de claves desde un seed BIP-39.

```
seed (BIP-39, 24 words)
  └─ sk (spending key) — ZIP-32 path: m/32'/133'/0'
       ├─ ask (spend authorizing key) — ToScalar(PRF(sk, [0x06]))
       ├─ nk  (nullifier key) — ToBase(PRF(sk, [0x07]))
       └─ rivk (raw incoming viewing key) — ToScalar(PRF(sk, [0x08]))
            └─ ak = [ask] × G_Orchard
                 └─ fvk = (ak, nk, rivk)
                      ├─ ivk = Commit_ivk(rivk, ak, nk)
                      ├─ ovk
                      ├─ dk (diversifier key)
                      └─ d (address) — derivado de dk
```

### Tests TDD (en orden)

```
test_generate_mnemonic()              — BIP-39 24 words, valid
test_seed_from_mnemonic()             — Mnemonic → 64-byte seed
test_spending_key_from_seed()         — seed → sk via ZIP-32 path m/32'/133'/0'
test_full_viewing_key_from_sk()       — sk → fvk = (ak, nk, rivk)
test_incoming_viewing_key_from_fvk()  — fvk → ivk
test_address_from_fvk()              — fvk → address d (Orchard)
test_deterministic_derivation()       — same seed → same keys always
test_ogbank_path_isolation()          — OGBank path (0x4F47') ≠ standard wallet path (0')
test_1_to_1_binding()                — one seed produces exactly one address d
```

### Struct principal

```rust
pub struct OGBankKeys {
    pub mnemonic: String,           // 24 words
    pub sk: SpendingKey,            // ZIP-32 derived
    pub fvk: FullViewingKey,        // (ak, nk, rivk)
    pub ivk: IncomingViewingKey,    // for trial decryption
    pub ovk: OutgoingViewingKey,    // for outgoing tx visibility
    pub address: Address,           // Orchard shielded address d
}

impl OGBankKeys {
    pub fn generate() -> Result<Self>;
    pub fn from_mnemonic(mnemonic: &str) -> Result<Self>;
    pub fn from_seed(seed: &[u8]) -> Result<Self>;
}
```

### Crates necesarios para esta fase

- `zcash_keys` — `UnifiedSpendingKey`, `UnifiedFullViewingKey`
- `zcash_protocol` — `ConsensusParameters`, network constants
- `orchard` — `SpendingKey`, `FullViewingKey`, `IncomingViewingKey`, `Address`
- `bip39` — Mnemonic generation

---

## Fase 2 — ogbank-core/crypto.rs — SK Encryption at Rest

**Referencia**: [08_mvp.md §SK Security](docs/technical/08_mvp.md)

### Qué implementar

Encriptación del spending key para almacenamiento en SQLite. ChaCha20-Poly1305 con clave derivada de Argon2id.

```
RELAYER_SK_PASSPHRASE (env var)
  → Argon2id(passphrase, random_salt) → 256-bit symmetric key
  → ChaCha20-Poly1305.Encrypt(key, nonce, sk_bytes)
  → stored: salt(16) || nonce(12) || ciphertext
```

### Tests TDD

```
test_encrypt_decrypt_roundtrip()     — encrypt(sk) → decrypt → original sk
test_wrong_passphrase_fails()        — decrypt with wrong passphrase → error
test_different_salt_different_output() — same sk, same passphrase → different ciphertext (random salt)
test_storage_format()                — output is salt(16) + nonce(12) + ciphertext
test_argon2id_params()               — verify Argon2id params (m=64MB, t=3, p=4)
test_zero_memory_after_use()         — sk bytes zeroed after encryption (zeroize crate)
```

### API

```rust
pub fn encrypt_sk(sk_bytes: &[u8], passphrase: &str) -> Result<Vec<u8>>;
pub fn decrypt_sk(encrypted: &[u8], passphrase: &str) -> Result<Zeroizing<Vec<u8>>>;
```

### Crates

- `chacha20poly1305` — AEAD encryption
- `argon2` — Key derivation from passphrase
- `rand` — Salt + nonce generation
- `zeroize` — Secure memory clearing

---

## Fase 3 — ogbank-core/scan.rs — Trial Decryption

**Referencia**: [05_zcash-integration.md §Trial Decryption](docs/technical/05_zcash-integration.md), [06_research.md §8a](docs/technical/06_research.md)

### Qué implementar

Dado un `ivk` y los encrypted outputs de una transacción Orchard, intentar decriptar cada Action para recuperar `(d, v, rseed, memo)`.

```
K_agree = [ivk] * epk                                      // ECDH on Pallas
K_sym   = BLAKE2b-256("Zcash_OrchardKDF", K_agree || epk)  // KDF
plaintext = ChaCha20-Poly1305.Decrypt(K_sym, C_enc)         // decrypt
if tag valid → parse (d, v, rseed, memo)
```

### Tests TDD

```
test_decrypt_own_note()              — given ivk + known encrypted note → recovers (v, memo)
test_decrypt_foreign_note_fails()    — different ivk → decryption fails silently (no error, just None)
test_commitment_verification()       — after decrypt, recompute cm and verify against on-chain cm
test_multiple_actions()              — tx with 5 actions, only 1 is ours → finds exactly 1
test_value_extraction()              — recovered v matches expected zatoshis
test_memo_extraction()               — 512-byte memo recovered correctly
```

### API

```rust
pub struct DecryptedNote {
    pub value_zat: u64,
    pub memo: [u8; 512],
    pub nullifier: Nullifier,
    pub commitment: NoteCommitment,
}

pub fn try_decrypt_action(
    ivk: &IncomingViewingKey,
    action: &OrchardAction,
) -> Option<DecryptedNote>;

pub fn scan_transaction(
    ivk: &IncomingViewingKey,
    tx_actions: &[OrchardAction],
) -> Vec<DecryptedNote>;
```

### Crates

- `orchard` — `Action`, `Note`, decryption primitives
- `zcash_note_encryption` — `try_note_decryption`

---

## Fase 4 — ogbank-cli — CLI Binary

**Referencia**: [08_mvp.md §Full Sequence](docs/technical/08_mvp.md)

### Qué implementar

CLI con 5 comandos. Usa `ogbank-core` para la criptografía.

### Comandos

| Comando | Acción | Depende de |
|---------|--------|-----------|
| `generate` | Crea mnemonic → keys → guarda en `~/.ogbank/keys.json` | Fase 1 (keys.rs) |
| `register` | POST /register a relayer con {sk, vk, ivk, address} | Fase 1 + Fase 5 |
| `scan --txid` | POST /scan/:id al relayer con {txid} | Fase 3 (scan.rs) + Fase 5 |
| `borrow --amount --recipient` | POST /borrow/:id al relayer | Fase 5 |
| `balance` | GET /balance/:id del relayer | Fase 5 |

### Tests TDD

```
test_generate_creates_keyfile()      — genera keys, verifica que ~/.ogbank/keys.json existe
test_generate_deterministic()        — from existing mnemonic → same keys
test_register_sends_correct_data()   — mock server, verifica payload
test_scan_parses_response()          — mock response, verifica output
test_borrow_validates_amount()       — amount <= 0 → error
test_balance_displays_correctly()    — mock response, verifica formato
```

### Estructura

```rust
#[derive(Parser)]
enum Commands {
    Generate,
    Register { #[arg(long)] relayer: String },
    Scan { #[arg(long)] relayer: String, #[arg(long)] txid: String },
    Borrow { #[arg(long)] relayer: String, #[arg(long)] amount: u64, #[arg(long)] recipient: String },
    Balance { #[arg(long)] relayer: String },
}
```

---

## Fase 5 — ogbank-relayer/db.rs — Database Schema

**Referencia**: [08_mvp.md §Data Model](docs/technical/08_mvp.md), [06_research.md §11d](docs/technical/06_research.md)

### Schema SQLite

```sql
CREATE TABLE positions (
    id TEXT PRIMARY KEY,
    address TEXT NOT NULL,                 -- zs1... (OGBank address)
    vk BLOB NOT NULL,                     -- full viewing key
    sk_encrypted BLOB NOT NULL,           -- ChaCha20-Poly1305 encrypted sk
    ivk BLOB NOT NULL,                    -- incoming viewing key
    balance_zat INTEGER DEFAULT 0,
    borrowed_zat INTEGER DEFAULT 0,
    note_count INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    position_id TEXT NOT NULL REFERENCES positions(id),
    txid TEXT NOT NULL,
    value_zat INTEGER NOT NULL,
    nullifier BLOB,
    memo BLOB,
    found_at TEXT NOT NULL,
    spent INTEGER DEFAULT 0
);

CREATE TABLE loans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    position_id TEXT NOT NULL REFERENCES positions(id),
    amount INTEGER NOT NULL,
    tx_hash TEXT NOT NULL,
    recipient TEXT NOT NULL,
    status TEXT DEFAULT 'active',          -- active / repaid / withdrawn
    created_at TEXT NOT NULL
);

CREATE TABLE withdrawals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    position_id TEXT NOT NULL REFERENCES positions(id),
    loan_id INTEGER NOT NULL REFERENCES loans(id),
    avax_tx_hash TEXT NOT NULL UNIQUE,
    block_number INTEGER NOT NULL,
    borrow_nullifier BLOB NOT NULL UNIQUE,
    repay_nullifier BLOB NOT NULL,
    amount_zat INTEGER NOT NULL,
    recipient_address TEXT NOT NULL,
    zec_tx_hash TEXT,
    zec_status TEXT DEFAULT 'pending',
    retry_count INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE event_cursor (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    last_block INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL
);
```

### Tests TDD

```
test_create_tables()                 — init_db succeeds, tables exist
test_insert_position()               — insert + read roundtrip
test_insert_note()                   — insert note linked to position
test_insert_loan()                   — insert loan, verify FK constraint
test_update_balance()                — update balance_zat, verify change
test_insert_withdrawal()             — insert withdrawal with all fields
test_withdrawal_idempotency()        — duplicate avax_tx_hash → UNIQUE violation
test_event_cursor_singleton()        — only one row allowed (CHECK constraint)
test_position_cascade()              — notes/loans reference valid position
```

---

## Fase 6 — ogbank-relayer/api.rs — REST API

**Referencia**: [08_mvp.md §Full Sequence](docs/technical/08_mvp.md)

### Endpoints

| Method | Path | Handler | Descripción |
|--------|------|---------|------------|
| POST | `/register` | `register_handler` | Registrar nueva posición (sk, vk, ivk, address) |
| POST | `/scan/:id` | `scan_handler` | Verificar depósito por txid (trial decryption) |
| POST | `/borrow/:id` | `borrow_handler` | Solicitar borrow (amount, recipient) |
| GET | `/balance/:id` | `balance_handler` | Consultar balance |
| POST | `/withdraw/:id` | `withdraw_handler` | Solicitar withdraw (proof, amount) |

### Tests TDD (integration con axum::test)

```
test_register_creates_position()     — POST /register → 201, position in DB
test_register_encrypts_sk()          — sk stored encrypted, not plaintext
test_scan_finds_deposit()            — mock Tatum response → deposit found
test_scan_no_match()                 — wrong txid → { found: false }
test_borrow_checks_collateral()      — balance < 150% of amount → 400 error
test_borrow_success()                — sufficient collateral → 200 + tx_hash
test_balance_returns_correct()       — inserted data → matches response
test_withdraw_triggers_zec_send()    — mock Zcash RPC → withdrawal created
```

---

## Fase 7 — ogbank-relayer/zcash.rs — Zcash RPC Client

**Referencia**: [05_zcash-integration.md](docs/technical/05_zcash-integration.md), [06_research.md §11b](docs/technical/06_research.md)

### Qué implementar

Cliente para Tatum API (fetching tx data) + Zcash node RPC (z_sendmany para MVP).

### API

```rust
pub struct TatumClient {
    base_url: String,
    api_key: String,
    http: reqwest::Client,
}

impl TatumClient {
    pub async fn get_transaction(&self, txid: &str) -> Result<ZcashTransaction>;
    pub async fn validate_address(&self, address: &str) -> Result<AddressInfo>;
}

pub struct ZcashNodeClient {
    rpc_url: String,
    http: reqwest::Client,
}

impl ZcashNodeClient {
    pub async fn z_sendmany(&self, from: &str, amounts: Vec<SendAmount>) -> Result<String>;
    pub async fn z_getoperationstatus(&self, opid: &str) -> Result<OperationStatus>;
    pub async fn z_importkey(&self, key: &str) -> Result<()>;
}
```

### Tests TDD

```
test_tatum_parse_transaction()       — mock JSON response → parsed ZcashTransaction
test_tatum_parse_orchard_actions()   — extract encrypted outputs from tx data
test_z_sendmany_request_format()     — verify JSON-RPC request body
test_z_sendmany_success()            — mock success response → opid returned
test_z_sendmany_failure()            — mock error → Result::Err with message
test_z_getoperationstatus_pending()  — status = "executing" → OperationStatus::Pending
test_z_getoperationstatus_success()  — status = "success" → OperationStatus::Success
test_z_getoperationstatus_failed()   — status = "failed" → OperationStatus::Failed
```

---

## Fase 8 — ogbank-relayer/evm.rs — EVM Signer

**Referencia**: [08_mvp.md §Phase 4: Borrow](docs/technical/08_mvp.md), [03_contracts.md](docs/technical/03_contracts.md)

### Qué implementar

Interacción con OGBankMVP.sol en Avalanche Fuji via `alloy`.

### API

```rust
pub struct EvmSigner {
    provider: Provider,
    signer: PrivateKeySigner,
    contract_address: Address,
}

impl EvmSigner {
    pub async fn borrow(
        &self,
        borrower: Address,
        collateral_zat: u64,
        amount: U256,
        position_id: &str,
    ) -> Result<TxHash>;

    pub async fn withdraw(
        &self,
        proof: Bytes,
        amount: U256,
    ) -> Result<TransactionReceipt>;
}
```

### Tests TDD

```
test_borrow_encodes_calldata()       — verify ABI encoding matches contract
test_borrow_signs_correctly()        — tx signed with correct private key
test_withdraw_returns_receipt()      — mock receipt with FinishPayment log
test_decode_finish_payment()         — extract (ogbank, amount, recipient, originAddress) from log
```

---

## Fase 9 — ogbank-relayer/listener.rs — Event Listener + ZEC Return

**Referencia**: [06_research.md §11](docs/technical/06_research.md), [09_responsibilities.md](docs/technical/09_responsibilities.md)

### Qué implementar

Background polling task + receipt watching para detectar FinishPayment y enviar ZEC.

### Flujo

```
Receipt watching (primary):
  1. withdraw_handler envía tx a Avalanche
  2. Espera receipt
  3. Decode FinishPayment de receipt.logs
  4. INSERT withdrawals (zec_status = 'pending')
  5. z_sendmany(escrow → originAddress, amount)
  6. UPDATE withdrawals (zec_status = 'sent', zec_tx_hash)

Polling fallback (background tokio::spawn):
  1. Read event_cursor.last_block
  2. eth_getLogs(fromBlock: last+1, toBlock: latest, topic: FinishPayment)
  3. For each: check idempotency → process → INSERT
  4. Update event_cursor
  5. Sleep 30s, repeat

Retry task (background):
  1. SELECT * FROM withdrawals WHERE zec_status = 'failed' AND retry_count < 5
  2. For each: z_sendmany retry
  3. Update status + retry_count
  4. After 5 retries: zec_status = 'requires_manual'
```

### Tests TDD

```
test_decode_finish_payment_from_receipt()  — real-format receipt → decoded event
test_polling_processes_new_events()        — mock eth_getLogs → events processed
test_polling_skips_processed_events()      — duplicate avax_tx_hash → skipped
test_polling_updates_cursor()              — after processing → last_block updated
test_retry_failed_withdrawal()             — zec_status=failed → retry → sent
test_retry_max_exceeded()                  — 5 retries → requires_manual
test_idempotency_receipt_and_polling()     — same event via both paths → processed once
```

---

## Fase 10 — Contratos Solidity (OGBankMVP.sol + MockUSDC.sol)

**Referencia**: [03_contracts.md](docs/technical/03_contracts.md), [08_mvp.md](docs/technical/08_mvp.md)

### OGBankMVP.sol

El MVP usa **relayer attestation** (no ZK proofs). El relayer es trusted para atestiguar el collateral.

```solidity
contract OGBankMVP {
    address public trustedRelayer;
    IERC20 public token; // MockUSDC

    mapping(bytes32 => bool) public positions; // positionId → exists
    mapping(bytes32 => uint256) public collateral; // positionId → collateral_zat

    function borrow(
        address borrower,
        uint256 collateralZat,
        uint256 amount,
        bytes32 positionId
    ) external onlyRelayer;

    event FinishPayment(
        address indexed ogbank,
        uint256 amount,
        address recipient,
        address originAddress
    );
}
```

### Tests Foundry

```
test_borrow_transfers_tokens()
test_borrow_only_relayer()
test_borrow_tracks_position()
test_borrow_requires_collateral_ratio()
```

---

## Fase 11 — Integration Tests E2E

### Test completo del ciclo

```
test_full_cycle():
  1. CLI: generate → mnemonic + keys
  2. CLI: register → position created in relayer DB
  3. (External) deposit ZEC to escrow address
  4. CLI: scan --txid → deposit found
  5. CLI: borrow --amount 100 --recipient 0xAlice → USDC transferred
  6. CLI: balance → shows 1.5 ZEC staked, 100 USDC borrowed
  7. (Future) repay + withdraw cycle
```

---

## Orden de ejecución

| Paso | Fase | Duración est. | Depende de |
|------|------|--------------|-----------|
| 1 | Fase 0: Scaffolding + skill | — | — |
| 2 | Fase 1: keys.rs | — | Fase 0 |
| 3 | Fase 2: crypto.rs | — | Fase 0 |
| 4 | Fase 3: scan.rs | — | Fase 1 |
| 5 | Fase 5: db.rs | — | Fase 0 |
| 6 | Fase 4: CLI | — | Fases 1-3 |
| 7 | Fase 6: api.rs | — | Fase 5 |
| 8 | Fase 7: zcash.rs | — | Fase 0 |
| 9 | Fase 8: evm.rs | — | Fase 0 |
| 10 | Fase 9: listener.rs | — | Fases 7-8 |
| 11 | Fase 10: Contracts | — | Fase 0 |
| 12 | Fase 11: E2E | — | Todas |

**Paralelizable**: Fases 1, 2, 5 pueden correrse en paralelo. Fases 7, 8, 10 pueden correrse en paralelo.

---

## Skill zlend — Patrones de código

La skill `zlend` se actualiza con estos patrones obligatorios:

### Error handling
```rust
// thiserror para library errors (ogbank-core)
#[derive(Debug, thiserror::Error)]
pub enum OGBankError {
    #[error("key derivation failed: {0}")]
    KeyDerivation(String),
    #[error("decryption failed: {0}")]
    Decryption(String),
    // ...
}

// anyhow para application errors (cli, relayer)
fn main() -> anyhow::Result<()> { ... }
```

### Testing pattern
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Deterministic test seed (NEVER use in production)
    const TEST_MNEMONIC: &str = "abandon abandon abandon ...";

    #[test]
    fn test_name_describes_behavior() {
        // Arrange
        let keys = OGBankKeys::from_mnemonic(TEST_MNEMONIC).unwrap();
        // Act
        let result = keys.ivk();
        // Assert
        assert!(result.is_some());
    }
}
```

### Serialization
```rust
// Hex encoding for all byte fields in JSON
#[derive(Serialize, Deserialize)]
struct ApiResponse {
    #[serde(with = "hex")]
    viewing_key: Vec<u8>,
    #[serde(with = "hex")]
    nullifier: Vec<u8>,
}
```

### Logging
```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(sk_encrypted))]
async fn register_position(
    db: &Database,
    position: &Position,
) -> Result<()> {
    info!(position_id = %position.id, "registering new position");
    // ...
}
```

### Configuration
```rust
#[derive(Clone)]
pub struct Config {
    pub relayer_port: u16,              // RELAYER_PORT, default 3000
    pub sk_passphrase: String,          // RELAYER_SK_PASSPHRASE (required)
    pub tatum_api_key: String,          // TATUM_API_KEY
    pub zcash_rpc_url: String,          // ZCASH_RPC_URL
    pub avalanche_rpc_url: String,      // AVALANCHE_RPC_URL
    pub contract_address: String,       // OGBANK_CONTRACT_ADDRESS
    pub relayer_private_key: String,    // RELAYER_PRIVATE_KEY (EVM signer)
}
```

---

## Research Note: Relayer Privacy — Data Cleartext Problem

**Problem**: The relayer holds `sk` (MVP) or FROST share 3 + `vk` (v1) in cleartext during operations. Even inside a TEE (SGX/TDX/SEV), data is unencrypted in the enclave — a side-channel attack or firmware vulnerability exposes everything. This is the deepest privacy risk in OGBank's architecture.

**NOT part of the implementation plan** — this is a research note for future hardening.

### Approaches analyzed

| Approach | Feasibility | Privacy gain | Performance cost |
|----------|:-----------:|:------------:|:----------------:|
| **FROST 2-of-3** (already planned v1) | High | High — relayer gets 1 share, can't sign alone | Low — 2 interactive rounds |
| **Shamir share in Zcash memo** | High | Same as FROST — share 3 in encrypted memo | Low |
| **FHE over trial decryption** | Very low | Total — relayer computes blind | Extreme — ECDH + BLAKE2b + ChaCha20 not FHE-friendly, 10,000x+ overhead |
| **MPC (2-party computation)** | Medium | High — neither party sees full data | Medium — latency per operation |
| **TEE + FROST** | High | Defense-in-depth — TEE isolates + FROST limits exposure | Low |
| **Threshold ECDH for ivk** | Medium | High — split viewing key operation across 2 parties | Medium — interactive protocol per scan |
| **ORAM inside TEE** | Low | Medium — hides memory access patterns from side-channel | High — 10-100x memory overhead |

### Recommendation path (not for MVP)

1. **v1**: FROST 2-of-3 (already planned) — eliminates full `sk` from relayer
2. **v1.5**: FROST + TEE — defense-in-depth, enclave isolates share 3
3. **v2**: Threshold ECDH for `ivk` — split the trial decryption across browser + relayer so neither alone can scan. The browser provides `ivk_share_1`, relayer holds `ivk_share_2`, they jointly compute `K_agree = [ivk] * epk` via threshold ECDH without either party learning `ivk` or `K_agree` individually
4. **v3**: MPC-based relayer — full multiparty computation for all sensitive operations

### Why FHE doesn't work here

Trial decryption requires:
```
K_agree = [ivk] * epk            ← elliptic curve scalar multiplication (Pallas)
K_sym = BLAKE2b-256(K_agree)      ← hash function
plaintext = ChaCha20-Poly1305(K_sym, C_enc)  ← symmetric cipher
```

FHE can handle additions and multiplications over encrypted data, but arbitrary hash functions (BLAKE2b) and stream ciphers (ChaCha20) require converting them to arithmetic circuits — millions of gates, making each trial decryption take minutes instead of microseconds. Not viable at 5,000 notes/sec scanning speed.

### Why Shamir/FROST IS the right answer

The user's intuition about Shamir sharing via Zcash memo is exactly what FROST implements, but better:
- Shamir SSS **reconstructs** the secret to sign → momentary exposure in memory
- FROST **never reconstructs** `ask` — partial signatures combine via algebra, the full key never exists in any single location after DKG
- Share 3 delivered to relayer via double-encrypted Zcash memo (NaCl inner + Orchard note outer) — already documented in [06_research.md §3b](docs/technical/06_research.md)

**Bottom line**: FROST solves the spending key problem. The remaining cleartext exposure is `vk` (viewing key) — solvable via threshold ECDH in v2, but the privacy loss from `vk` is observation-only (balance visibility), not fund theft. Acceptable risk for v1.

---

## Archivos a crear/modificar

| Archivo | Acción | Fase |
|---------|--------|------|
| `docs/technical/10_implementation-plan.md` | **Crear** — este plan completo | — |
| `MVP-POC/Cargo.toml` | **Crear** — workspace root | 0 |
| `MVP-POC/crates/ogbank-core/src/lib.rs` | **Crear** | 0 |
| `MVP-POC/crates/ogbank-core/src/keys.rs` | **Crear** — ZIP-32 derivation | 1 |
| `MVP-POC/crates/ogbank-core/src/crypto.rs` | **Crear** — SK encryption | 2 |
| `MVP-POC/crates/ogbank-core/src/scan.rs` | **Crear** — Trial decryption | 3 |
| `MVP-POC/crates/ogbank-cli/src/main.rs` | **Crear** — CLI | 4 |
| `MVP-POC/crates/ogbank-relayer/src/db.rs` | **Crear** — SQLite schema | 5 |
| `MVP-POC/crates/ogbank-relayer/src/api.rs` | **Crear** — REST endpoints | 6 |
| `MVP-POC/crates/ogbank-relayer/src/zcash.rs` | **Crear** — Tatum + Zcash RPC | 7 |
| `MVP-POC/crates/ogbank-relayer/src/evm.rs` | **Crear** — alloy EVM signer | 8 |
| `MVP-POC/crates/ogbank-relayer/src/listener.rs` | **Crear** — Event poller + ZEC send | 9 |
| `MVP-POC/crates/ogbank-relayer/src/main.rs` | **Crear** — axum server startup | 6 |
| `MVP-POC/contracts/src/OGBankMVP.sol` | **Crear** — MVP contract | 10 |
| `MVP-POC/contracts/src/MockUSDC.sol` | **Crear** — Test token | 10 |
| `MVP-POC/contracts/test/OGBankMVP.t.sol` | **Crear** — Foundry tests | 10 |
| `MVP-POC/contracts/script/Deploy.s.sol` | **Crear** — Deploy script | 10 |

---

## Verificación

Para cada fase:
1. `cargo test -p ogbank-core` (o el crate correspondiente) — todos los tests pasan
2. `cargo clippy` — sin warnings
3. `cargo fmt --check` — formateado

Para contratos:
4. `forge test` — todos los tests pasan

E2E:
5. Relayer corriendo en localhost:3000
6. `ogbank-cli generate` → keys generadas
7. `ogbank-cli register` → position en DB
8. Depositar ZEC en testnet
9. `ogbank-cli scan --txid ...` → deposit found
10. `ogbank-cli borrow --amount 100 --recipient 0x...` → tokens transferidos

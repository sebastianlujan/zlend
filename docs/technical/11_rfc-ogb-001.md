# RFC-OGB-001: OGBank Protocol Specification

```
Title:    OGBank — Delegated Spending Vaults for ZCash Orchard
Version:  1.0-draft
Author:   Sebastian Lujan, Franco Perez @ OGBank
Status:   Draft
Created:  2026-03-08:
License:  MIT
Requires: ZIP-32, ZIP-224, ZIP-302, ZIP-312
```

---

## Abstract

OGBank defines a protocol for creating delegated spending vaults in the ZCash Orchard shielded pool. A vault is a ZIP-32-derived Orchard sub-account whose spend authorizing key (`ask`) is threshold-shared via Re-Randomized FROST (ZIP-312) in a 2-of-3 configuration between an owner (2 shares) and a relayer (1 share). The owner pre-authorizes spending by issuing single-use Authorization Tickets — cryptographic commitments binding specific spending parameters (amount, destination, expiry) — organized in a Merkle tree with nullifier-based replay prevention, following the commitment-nullifier pattern established by Tornado Cash. An automated signer daemon holds one owner share and validates relayer requests against the ticket tree deterministically, enabling the owner to remain offline between ticket batch creation sessions. An oblivious nullifier watch service enables chain synchronization without exposing the vault's viewing key to the relayer. On-chain, all vault operations are indistinguishable from standard Orchard transactions.

---

## 1. Terminology

| Term | Definition |
|---|---|
| **Vault** | An Orchard sub-account whose `ask` is FROST-shared |
| **Owner** | The entity that controls the vault's funds and creates authorization tickets |
| **Relayer** | A service authorized to initiate spends from the vault within ticket constraints |
| **Signer** | An automated daemon holding one owner FROST share, validating signing requests |
| **Authorization Ticket** | A single-use pre-commitment to specific spending parameters |
| **Auth Tree** | Merkle tree of authorization ticket commitments |
| **Auth Nullifier** | Value revealed when a ticket is consumed, preventing replay |
| **NullifierWatch** | Service that monitors the chain for vault note nullifiers without learning note details |
| **PCD** | Proof-Carrying Data — wallet state accompanied by a proof of its own correctness |

**Key words**: "MUST", "MUST NOT", "SHALL", "SHOULD", "MAY" are interpreted as in RFC 2119.

---

## 2. Motivation

ZCash Orchard provides binary spend authorization: possession of `ask` grants unlimited spending power over an address. There is no native mechanism for delegated, bounded, or conditional spending — no equivalent to ERC-20 `approve/transferFrom`. OGBank introduces this capability at the application layer without protocol changes, enabling use cases including:

- DeFi collateral lockup (shielded ZEC as collateral on Ethereum-based lending protocols)
- Shielded treasury management with role-based access
- Escrow with automated dispute resolution
- Payroll systems with bounded recurring payments

---

## 3. System Architecture

### 3.1 Participants

The system has four participant roles:

**Owner A (Hot Device)**: Holds FROST KeyPackage₁ (share 1 of `ask`). Manages vault lifecycle, creates authorization ticket batches, provisions the Signer. Online during setup and periodic ticket renewal.

**Owner B (Cold Storage)**: Holds FROST KeyPackage₂ (share 2 of `ask`). Used only for emergency recovery (Owner A + Owner B can spend without relayer). SHOULD be a hardware wallet or air-gapped device.

**Relayer**: Holds FROST KeyPackage₃ (share 3 of `ask`). Constructs transactions, coordinates FROST signing ceremonies, bridges to external chains. Online continuously. Cannot spend unilaterally (holds only 1-of-3).

**Automated Signer**: A daemon process controlled by Owner A. Holds KeyPackage₁, the Auth Tree root, pre-committed nonces, and the vault's note state. Validates relayer signing requests against the authorization ticket tree. MUST run continuously for delegated spending to function. If the Signer goes offline, delegated spending pauses but funds remain safe.

### 3.2 Trust Model

```
Relayer compromised:        Attacker holds 1-of-3 shares. Cannot spend.
                            Maximum exposure = Σ(outstanding ticket amounts).
                            Owner sweeps vault using shares 1+2.

Signer compromised:         Attacker holds 1-of-3 shares + auth tree state.
                            Can approve fraudulent signing requests.
                            Owner revokes by not issuing new tickets,
                            sweeps vault using shares 1+2 (if Owner B available).

Relayer + Signer:           Attacker holds 2-of-3 shares. CAN SPEND.
                            This is the fundamental threshold assumption.
                            Mitigate: Owner B on hardware wallet.

Owner A + Owner B:          Can spend without relayer (recovery).
                            Can sweep vault, revoke all delegation.

All three compromised:      Total loss. Same as any 3-of-3 scheme.

NullifierWatch compromised: Learns which 32-byte values to watch (opaque).
                            Cannot determine note amounts, addresses, or
                            link nullifiers to note commitments.
                            Privacy impact: negligible.
```

### 3.3 Architectural Layers

```
┌───────────────────────────────────────────────────────┐
│ Layer 4: Cross-Chain                                  │
│   Balance PCD proof → Ethereum verifier → Aave        │
├───────────────────────────────────────────────────────┤
│ Layer 3: Relayer Logic                                │
│   TX construction, FROST coordination, batch spending │
├───────────────────────────────────────────────────────┤
│ Layer 2: Authorization                                │
│   Ticket commitments, Auth Tree, nullifier tracking   │
├───────────────────────────────────────────────────────┤
│ Layer 1: Vault                                        │
│   ZIP-32 derivation, FROST key ceremony, note mgmt   │
└───────────────────────────────────────────────────────┘
```

---

## 4. Layer 1: Vault

### 4.1 Vault Account Derivation

A vault is an Orchard sub-account derived via ZIP-32 hardened-only key derivation.

**Derivation path**: `m_Orchard / 32' / 133' / vault_account_index'`

The `vault_account_index` is computed as:

```
vault_account_index = BLAKE2b-256("OGBank__VaultIdx", seed_fingerprint || I2LEOSP_32(vault_nonce)) mod 2^31
```

Where:
- `seed_fingerprint` is the ZIP-32 seed fingerprint of the owner's master seed (BLAKE2b-256 of the seed, per ZIP-32 §4.1)
- `vault_nonce` is a 32-bit unsigned integer incremented for each new vault under the same seed
- The domain separator `"OGBank__VaultIdx"` is the BLAKE2b personalization parameter (16 bytes)

Implementations MUST reject `vault_account_index` values that produce an invalid Orchard spending key (per ZIP-32 §5.4.2.1) and increment `vault_nonce` until a valid key is obtained.

### 4.2 Key Structure

From the derived spending key `sk_vault`, the Orchard key tree produces:

```
sk_vault (32 bytes)
├── ask  = ToScalar(PRF^expand(sk_vault, [0x06]))      Spend Authorizing Key
├── nk   = ToBase(PRF^expand(sk_vault, [0x07]))         Nullifier Deriving Key
├── rivk = ToScalar(PRF^expand(sk_vault, [0x08]))        Commitment Randomness Key
│
├── ak   = [ask] · G^Orchard                             Spend Validating Key (public)
├── fvk  = (ak, nk, rivk)                                Full Viewing Key
│   ├── dk  = PRF^expand(rivk, [0x82] || ak || nk)[0..32]
│   ├── ovk = PRF^expand(rivk, [0x82] || ak || nk)[32..64]
│   └── ivk = SinsemillaShortCommit_rivk(ak, nk)         Incoming Viewing Key
│
└── addr = (d, pk_d)                                      Default Payment Address
    ├── d   = first valid diversifier from dk
    └── pk_d = [ivk] · DiversifyHash(d)
```

**In OGBank, `ask` is replaced by FROST shares.** The spending key `sk_vault` is used only to derive `nk` and `rivk`. After extracting these values, `sk_vault` and `ask` MUST be securely erased from memory. The `ak` used in `fvk` is the FROST group public key, not `[ask] · G`.

### 4.3 FROST Key Ceremony

OGBank uses a 2-of-3 Re-Randomized FROST configuration (ZIP-312) over the Pallas curve with the `PallasBlake2b512` ciphersuite.

#### 4.3.1 Distributed Key Generation (DKG) — RECOMMENDED

Participants: Owner A (id=1), Owner B (id=2), Relayer (id=3). Threshold: t=2, Total: n=3.

**Round 1**: Each participant P_i generates a degree-(t-1) = degree-1 polynomial `f_i(x) = a_{i,0} + a_{i,1} · x` over the Pallas scalar field. They compute commitments `C_{i,k} = [a_{i,k}] · G^Orchard` for k ∈ {0,1} and a Schnorr proof of knowledge of `a_{i,0}`. Each participant broadcasts their `Round1Package = (C_{i,0}, C_{i,1}, proof_i)` to all other participants and retains `Round1SecretPackage = (a_{i,0}, a_{i,1})` privately.

**Round 2**: Each participant P_i evaluates their polynomial at each other participant's identifier: `s_{i→j} = f_i(id_j)`. They send `Round2Package(i→j) = s_{i→j}` to participant P_j over a confidential and authenticated channel.

**Round 3**: Each participant P_j verifies all received Round1Packages (checking Schnorr proofs and commitment consistency), then computes their secret share: `share_j = Σ_i s_{i→j}`. They verify: `[share_j] · G = Σ_i Σ_k [C_{i,k} · id_j^k]`. The group public key is: `ak = Σ_i C_{i,0}`.

Output per participant:
```
KeyPackage_j = {
    identifier:      id_j,
    signing_share:   share_j,           // SECRET — this is the ask share
    verifying_share: [share_j] · G,
    verifying_key:   ak,                // Same for all participants
    min_signers:     2,
}

PublicKeyPackage = {
    verifying_shares: {id_1: VS_1, id_2: VS_2, id_3: VS_3},
    verifying_key:    ak,               // This is the vault's SpendValidatingKey
}
```

#### 4.3.2 Trusted Dealer — PERMITTED for migration

If migrating an existing wallet to OGBank, the Trusted Dealer path MAY be used:

1. Derive `sk_vault` and extract `ask` per §4.2
2. Call `frost::keys::generate_with_dealer(3, 2, IdentifierList::Default, rng)`
3. Distribute KeyPackages to each participant over secure channels
4. Securely erase `sk_vault`, `ask`, and the dealer's memory

The Trusted Dealer represents a single point of failure during the ceremony. DKG (§4.3.1) is RECOMMENDED for new vaults.

#### 4.3.3 Vault Assembly

After the key ceremony completes:

1. The FROST group public key `ak` is extracted from `PublicKeyPackage.verifying_key()`
2. Owner A derives `nk` and `rivk` from `sk_vault` per §4.2
3. The vault's FVK is assembled: `fvk = (ak_frost, nk, rivk)`
4. The vault's payment address is derived from `fvk`
5. `sk_vault` and `ask` are securely erased
6. A unique vault identifier is computed:
   ```
   vault_id = BLAKE2b-256("OGBank__VaultID_", ak || I2LEOSP_32(vault_nonce))
   ```

### 4.4 Key Distribution

After vault assembly, keys are distributed as follows:

| Component | Owner A | Owner B | Relayer | Signer |
|---|---|---|---|---|
| KeyPackage₁ (ask share 1) | ✓ | | | ✓ |
| KeyPackage₂ (ask share 2) | | ✓ | | |
| KeyPackage₃ (ask share 3) | | | ✓ | |
| PublicKeyPackage | ✓ | ✓ | ✓ | ✓ |
| nk (nullifier deriving key) | ✓ | | | ✓ |
| rivk | ✓ | | | ✓ |
| fvk = (ak, nk, rivk) | ✓ | | | ✓ |
| ivk (incoming viewing key) | ✓ | | | ✓ |
| Vault address | ✓ | ✓ | ✓ | ✓ |

The Relayer MUST NOT receive `nk`, `rivk`, `fvk`, or `ivk`. The Relayer receives only its own KeyPackage, the PublicKeyPackage, and the vault's payment address. Note details required for transaction construction are provided by the Signer on a per-spend basis (see §7.4).

### 4.5 Vault Funding

Deposits are standard Orchard shielded transactions to the vault's payment address. The sender MUST communicate the note details (rseed, value) to the Signer out-of-band using the Vault Payment Request protocol (§6.4). The Signer adds received note details to its local UTXO set.

---

## 5. Layer 2: Authorization

### 5.1 Authorization Ticket

An Authorization Ticket is a single-use pre-commitment to a bounded spend. Each ticket authorizes at most one transaction from the vault.

#### 5.1.1 Ticket Structure

```
AuthorizationTicket = {
    auth_secret:       [u8; 32],    // Random, shared with Relayer
    auth_nullifier:    [u8; 32],    // Derived from auth_secret
    max_amount:        u64,         // Maximum zatoshi for this ticket
    destination_hash:  [u8; 32],    // H(recipient_address)
    expiry_block:      u32,         // Block height after which ticket is invalid
    nonce_commitment:  [u8; 64],    // Pre-generated FROST nonce commitment (hiding || binding)
}
```

#### 5.1.2 Ticket Generation

For each ticket, the Owner:

1. Samples `auth_secret ←$ {0,1}^256` uniformly at random
2. Derives `auth_nullifier = BLAKE2b-256("OGBank_AuthNull_", auth_secret)`
3. Computes `destination_hash = BLAKE2b-256("OGBank_AuthDest", recipient_address_bytes)`
4. Pre-generates a FROST nonce pair for KeyPackage₁:
   ```
   (nonces_i, commitments_i) = frost::round1::commit(key_package.signing_share(), rng)
   ```
5. Computes the ticket commitment (§5.2)

#### 5.1.3 Nullifier Hash

The value publicly revealed when a ticket is consumed:

```
auth_nullifier_hash = BLAKE2b-256("OGBank_NullHash", auth_nullifier)
```

This is a double-hash: `H(H(auth_secret))`. The Relayer knows `auth_secret` (shared by Owner) and can compute `auth_nullifier` and `auth_nullifier_hash`, but revealing `auth_nullifier_hash` does not expose `auth_secret` to third parties.

### 5.2 Authorization Commitment

Each ticket produces a commitment leaf for the Auth Tree:

```
commitment = BLAKE2b-256(
    "OGBank_AuthComm_",
    auth_nullifier || auth_secret || I2LEOSP_64(max_amount) ||
    destination_hash || I2LEOSP_32(expiry_block)
)
```

Properties:
- **Hiding**: Without `auth_secret`, the commitment cannot be opened
- **Binding**: The commitment uniquely determines all ticket parameters
- **Collision-resistant**: Finding two tickets with the same commitment is computationally infeasible

### 5.3 Authorization Merkle Tree

The Auth Tree is a fixed-depth binary Merkle tree over ticket commitments.

**Depth**: 20 (supports up to 2^20 = 1,048,576 tickets per vault lifecycle)

**Hash function**: `BLAKE2b-256("OGBank_TreeNode", left || right)`

**Empty leaf**: `BLAKE2b-256("OGBank_TreeLeaf", [0u8; 32])` (constant, precomputed)

**Zero hashes**: For each level `l`, `zero_hash(l)` is computed recursively:
```
zero_hash(0) = empty_leaf
zero_hash(l) = BLAKE2b-256("OGBank_TreeNode", zero_hash(l-1) || zero_hash(l-1))
```

Tickets are inserted left-to-right. The tree root is updated incrementally after each insertion.

### 5.4 Authorization Merkle Proof

A proof that commitment `c` exists at leaf index `idx` in the Auth Tree with root `R`:

```
AuthMerkleProof = {
    leaf_index: u32,
    path:       [(sibling_hash: [u8; 32], is_left: bool); TREE_DEPTH],
}
```

Verification: starting from `current = c`, for each level `(sibling, is_left)`:
```
if is_left:
    current = H(current || sibling)
else:
    current = H(sibling || current)
```
Accept if `current == R`.

### 5.5 Spent Nullifier Set

The Signer maintains a Sparse Merkle Tree (SMT) of depth 256 tracking consumed ticket nullifiers.

**SMT hash**: `BLAKE2b-256("OGBank_SMTNode_", left || right)`

**Empty value**: `[0u8; 32]`

Operations:
- **Insert**: Insert `auth_nullifier_hash` at the key position determined by interpreting the hash as a 256-bit path. Update the SMT root.
- **Non-membership proof**: Prove that a key maps to the empty value. Proof size: 256 hashes (compressible with common-prefix optimization).
- **Membership check**: Prove that a key maps to a non-empty value.

The SMT root is a single 32-byte hash committing to the entire spent-nullifier state.

### 5.6 Ticket Batch Creation

The Owner creates tickets in batches during online sessions.

**Inputs**: Spending parameters (max_amount, destination, expiry), batch count N.

**Process**:
1. For i = 1..N: generate ticket per §5.1.2
2. Insert all commitments into the Auth Tree (§5.3)
3. Package the Relayer payload:
   ```
   RelayerTicketBatch = {
       tickets: [{auth_secret, max_amount, destination_hash, expiry_block,
                  nonce_commitment}; N],
       merkle_tree: full tree structure (for proof computation),
       auth_tree_root: [u8; 32],
   }
   ```
4. Package the Signer payload:
   ```
   SignerTicketBatch = {
       auth_tree_root: [u8; 32],
       valid_nullifier_hashes: [[u8; 32]; N],
       nonce_registry: {auth_nullifier_hash → SigningNonces; N},
       ticket_params: [{max_amount, destination_hash, expiry_block}; N],
   }
   ```
5. Deliver RelayerTicketBatch to Relayer over authenticated encrypted channel
6. Deliver SignerTicketBatch to Signer (local, or over authenticated encrypted channel)

---

## 6. Oblivious Chain Synchronization

### 6.1 Design Rationale

The Relayer MUST NOT receive the vault's FVK, IVK, or `nk`. Instead, chain synchronization is split between the Signer (who holds the FVK) and a NullifierWatch service (which learns only opaque 32-byte values).

This design draws on the "oblivious synchronization" concept from Tachyon (Sean Bowe, 2025): a third-party service can track wallet state by monitoring nullifiers without learning anything about the underlying notes, because nullifiers are unlinkable to note commitments by the Spend Unlinkability property of Orchard.

### 6.2 NullifierWatch Service

The NullifierWatch service is a lightweight chain-monitoring process.

**Provisioning**: The Signer sends the service a set of nullifier values to watch:
```
WatchRequest = {
    vault_id:    [u8; 32],
    nullifiers:  Vec<[u8; 32]>,    // Opaque 32-byte values
    start_block: u32,               // Start monitoring from this height
}
```

**Monitoring**: For each new block, the service compares the block's revealed nullifier set against the watched set.

**Reporting**: When a match is found:
```
NullifierAlert = {
    vault_id:     [u8; 32],
    nullifier:    [u8; 32],
    block_height: u32,
    block_hash:   [u8; 32],
}
```

**Privacy properties**: The service learns:
- That the Signer is interested in certain 32-byte values
- When those values appear on-chain

The service does NOT learn:
- Which note commitments correspond to these nullifiers
- The amounts, addresses, or any other note details
- Whether the nullifiers belong to the same wallet or different wallets

**Trust requirement**: None beyond availability. The service can be the Relayer itself (since nullifiers reveal nothing), a public lightwalletd instance, or any untrusted third party.

### 6.3 Signer Chain State Management

The Signer maintains the vault's UTXO set:

```
VaultState = {
    unspent_notes: Vec<{
        note:        OrchardNote,       // (d, pk_d, v, ρ, ψ, rcm)
        commitment:  [u8; 32],          // Note commitment in tree
        merkle_path: MerklePath,        // Path to note in commitment tree
        nullifier:   [u8; 32],          // Precomputed: DeriveNullifier_nk(ρ, ψ, cm)
        received_at: u32,               // Block height when note was received
    }>,
    total_balance:   u64,               // Sum of unspent note values (zatoshi)
    last_synced:     u32,               // Block height of last sync
    commitment_tree: IncrementalMerkleTree, // For computing fresh Merkle paths
}
```

**Sync process**:
1. Receive NullifierAlerts from NullifierWatch (§6.2)
2. For each alert: mark the corresponding note as spent, remove from `unspent_notes`
3. Receive note details from Owner out-of-band (§6.4) when new deposits arrive
4. Update `commitment_tree` from public chain data (commitment tree updates are public)
5. Refresh Merkle paths for all unspent notes

### 6.4 Out-of-Band Note Distribution

When the vault receives a deposit, the Signer needs the note plaintext to add it to VaultState. Since the Signer holds the IVK, it CAN trial-decrypt outputs. However, to minimize chain scanning, the Owner SHOULD communicate note details directly.

**Vault Payment Request** (for third-party deposits):
```
VaultPaymentRequest = {
    vault_address:   OrchardAddress,
    ephemeral_pk:    X25519PublicKey,   // For encrypting note details back to Signer
    callback_url:    Option<String>,    // Where to send encrypted note details
    amount_hint:     Option<u64>,       // Requested amount (informational)
}
```

The depositor encrypts `(rseed, value)` to `ephemeral_pk` and delivers the ciphertext to the Signer via `callback_url` or a store-and-forward service. The Signer decrypts, derives the full note, and adds it to VaultState.

**Owner deposits** (self-funding): The Owner's wallet software sends note details to the Signer directly as part of the deposit transaction workflow.

---

## 7. Layer 3: Relayer Logic

### 7.1 Relayer State

```
RelayerState = {
    vault_id:            [u8; 32],
    vault_address:       OrchardAddress,
    frost_key_package:   KeyPackage,        // KeyPackage₃ (relayer's ask share)
    frost_pubkey_package: PublicKeyPackage,

    // Authorization ticket inventory
    available_tickets:   Vec<AuthorizationTicket>,
    consumed_tickets:    Vec<[u8; 32]>,     // auth_nullifier_hashes
    auth_tree:           AuthorizationTree, // Full tree for proof computation

    // Nonce pool for relayer's own FROST signing
    nonce_pool:          Vec<(SigningNonces, SigningCommitments)>,

    // Cached vault state (provided by Signer per §7.4)
    spendable_notes:     Option<SpendableNoteSet>,
}
```

### 7.2 Signing Ceremony (Re-Randomized FROST)

For each Orchard Action that spends a vault note, a FROST signing ceremony produces a valid RedPallas SpendAuthSig.

#### 7.2.1 Nonce Pre-Processing

Both Relayer and Signer pre-generate nonce pools.

**Relayer nonces**: Generated in batches using `frost::round1::commit(signing_share, rng)`. Stored as `(SigningNonces, SigningCommitments)` pairs. Nonces are SINGLE-USE. After use, they MUST be securely erased.

**Signer nonces**: Pre-committed during ticket batch creation (§5.6). Each nonce pair is bound to exactly one authorization ticket via `nonce_registry[auth_nullifier_hash]`.

#### 7.2.2 Transaction Construction

The Relayer constructs the unsigned Orchard transaction:

1. Receive `SpendableNoteSet` from Signer (§7.4)
2. Select input notes whose total value ≥ required amount + fee
3. Construct Action descriptions:
   - **Spend side**: nullifier (provided by Signer), rk (placeholder), SpendAuthSig (TBD)
   - **Output side**: note commitment, ephemeral key, encrypted note, encrypted outgoing
4. Compute value commitments: `cv = [v] · V + [rcv] · R`
5. Package the unsigned transaction and send to Signer for proof generation

#### 7.2.3 Proof Generation by Signer

The Signer generates the Halo2 ZK proof because it holds `nk` (required as a private witness in the Orchard Action circuit).

The Signer receives the unsigned transaction structure from the Relayer and:

1. Inserts `nk` as the private witness for nullifier integrity
2. Inserts the Merkle path for each spent note
3. Generates the randomizer `α` for each action: `α ←$ F_r` (Pallas scalar field)
4. Computes `rk = [ask_frost_group_key + α] · G` (the randomized validating key)
5. Generates the Halo2 proof over the Vesta curve
6. Computes the SIGHASH:
   ```
   SIGHASH = BLAKE2b-256("ZcashTxHash_V5",
       header_digest || transparent_digest || sapling_digest || orchard_digest)
   ```
7. Returns to Relayer: `(proof, rk, SIGHASH, randomizer)` per action

#### 7.2.4 FROST Signing

For each action requiring a SpendAuthSig:

**Relayer** (Coordinator):
1. Collects nonce commitments: own (from pool) + Signer's (from ticket)
2. Builds `commitment_list = [(id_1, signer_hiding, signer_binding), (id_3, relayer_hiding, relayer_binding)]`
3. Builds `SigningPackage = (commitment_list, SIGHASH)`
4. Computes the FROST randomizer from `α`:
   ```
   randomizer = frost_rr::Randomizer::from_scalar(α)
   ```
5. Sends `(SigningPackage, randomizer)` to Signer along with the AuthorizationProof

**Signer**:
1. Validates the authorization (§7.3)
2. Produces re-randomized signature share:
   ```
   z_signer = frost_rr::sign(&signing_package, &nonces, &key_package, &randomizer)
   ```
3. Returns `SignatureShare` to Relayer

**Relayer**:
1. Produces own share: `z_relayer = frost_rr::sign(&signing_package, &own_nonces, &key_package, &randomizer)`
2. Aggregates: `SpendAuthSig = frost_rr::aggregate(&signing_package, &shares, &pubkey_package, &randomizer)`
3. Verifies locally: `RedPallas.Verify(rk, SIGHASH, SpendAuthSig)`
4. Attaches SpendAuthSig to the action description
5. Computes BindingSig: `bsk = Σ rcv_i`, `BindingSig = RedPallas.Sign(bsk, SIGHASH)`
6. Broadcasts the complete transaction to the ZCash network

### 7.3 Authorization Validation (Signer-Side)

When the Signer receives a `SigningRequest`, it performs seven deterministic checks. ALL MUST pass for the Signer to release a signature share.

```
SigningRequest = {
    sighash:              [u8; 32],
    randomizer:           Randomizer,
    signing_commitments:  BTreeMap<Identifier, SigningCommitments>,
    auth_proof:           AuthorizationProof,
    proposed_tx:          ProposedTransaction,
}

AuthorizationProof = {
    auth_secret:          [u8; 32],
    auth_nullifier_hash:  [u8; 32],
    max_amount:           u64,
    destination_hash:     [u8; 32],
    expiry_block:         u32,
    merkle_proof:         AuthMerkleProof,
}
```

**CHECK 1 — Commitment Integrity**: Reconstruct the commitment from the proof's parameters:
```
auth_nullifier = BLAKE2b-256("OGBank_AuthNull_", auth_proof.auth_secret)
expected_commitment = BLAKE2b-256("OGBank_AuthComm_",
    auth_nullifier || auth_proof.auth_secret || I2LEOSP_64(auth_proof.max_amount) ||
    auth_proof.destination_hash || I2LEOSP_32(auth_proof.expiry_block))
```
Verify `expected_commitment` matches the leaf in the Merkle proof.

**CHECK 2 — Merkle Proof Validity**: Verify `auth_proof.merkle_proof` against the stored `auth_tree_root` using the expected commitment as the leaf.

**CHECK 3 — Replay Prevention**: Verify `auth_proof.auth_nullifier_hash` is NOT in the spent nullifier SMT (§5.5).

**CHECK 4 — Nullifier Derivation**: Verify:
```
expected_nullifier_hash = BLAKE2b-256("OGBank_NullHash", auth_nullifier)
assert(expected_nullifier_hash == auth_proof.auth_nullifier_hash)
```

**CHECK 5 — Expiry**: Verify `current_block_height ≤ auth_proof.expiry_block`.

**CHECK 6a — Amount Bound**: Verify `proposed_tx.total_spend_value ≤ auth_proof.max_amount`.

**CHECK 6b — Destination Match**: Verify:
```
tx_destination_hash = BLAKE2b-256("OGBank_AuthDest", proposed_tx.primary_recipient_bytes)
assert(tx_destination_hash == auth_proof.destination_hash)
```

**CHECK 7 — SIGHASH Consistency**: Independently compute the SIGHASH from `proposed_tx` and verify it equals `signing_request.sighash`.

**On success**: Mark `auth_nullifier_hash` as spent in the SMT. Retrieve the pre-committed nonce from `nonce_registry[auth_nullifier_hash]`. Produce the signature share. Return it to the Relayer.

**On failure**: Return the specific error. Do NOT mark the nullifier as spent. Do NOT produce a signature share.

### 7.4 Note Provision to Relayer

The Signer provides note details to the Relayer only for authorized spends:

```
SpendableNoteSet = {
    notes: Vec<{
        note_plaintext:  OrchardNotePlaintext,   // (d, pk_d, v, rseed, memo)
        merkle_path:     MerklePath,              // Path in commitment tree
        nullifier:       [u8; 32],                // Pre-computed by Signer
    }>,
    anchor:          [u8; 32],    // Current Orchard anchor (commitment tree root)
    as_of_block:     u32,         // Block height of this snapshot
}
```

The Relayer receives this snapshot ONLY when requesting a spend. It DOES NOT receive historical transaction data, balance information beyond what's needed for the current spend, or note details for notes not involved in the current transaction.

### 7.5 Batch Spending

The Relayer MAY consume multiple authorization tickets in a single FROST signing session to package multiple spends into one transaction with multiple Orchard Actions.

For N spends in a single transaction:
- N authorization tickets are consumed (N nullifiers marked spent)
- N signature shares are produced (one per action)
- N Halo2 proofs are generated (batched by the Signer)
- 1 BindingSig covers the entire transaction
- 1 fee is paid (ZIP-317 fee scales with action count but has one base cost)

---

## 8. Layer 4: Cross-Chain Balance Attestation

### 8.1 Balance PCD Proof

The Signer generates a proof asserting the vault's balance without revealing transaction history.

**Public inputs**:
```
vault_id_commitment:  [u8; 32],    // Hiding commitment to vault identity
orchard_anchor:       [u8; 32],    // Commitment tree root at block B
smt_nullifier_root:   [u8; 32],    // Zcash nullifier set root at block B
claimed_min_balance:  u64,          // "Balance ≥ this value"
block_height:         u32,          // Block B
```

**Private witnesses**:
```
fvk:                 FullViewingKey,
notes:               Vec<(OrchardNote, MerklePath)>,  // Unspent notes
nullifiers:          Vec<([u8; 32], NullifierNonMembershipProof)>,
```

**Circuit constraints**:
1. For each note: note commitment is in the tree (Merkle path valid against `orchard_anchor`)
2. For each note: `ivk` derived from `fvk` correctly decrypts the note (verifies `pk_d = [ivk] · g_d`)
3. For each note: the nullifier derived from `nk` and the note is NOT in the nullifier set (non-membership proof against `smt_nullifier_root`)
4. `Σ(note values) ≥ claimed_min_balance`
5. `vault_id_commitment` binds to the `fvk`

**Implementation**: This circuit can be built in Halo2 using the existing Orchard gadgets (Sinsemilla for commitments, Poseidon for nullifiers, Merkle path verification). For Ethereum verification, the proof is either re-proven in a BN254-friendly system or wrapped using proof composition.

### 8.2 Ethereum Verifier Contract

```solidity
// OGBankOracle.sol (simplified)

interface IOGBankOracle {
    /// @notice Verify a PCD balance proof and update attested balance
    /// @param vaultId Unique vault identifier
    /// @param minBalance Minimum balance proven (in zatoshi)
    /// @param blockHeight ZCash block height of the proof
    /// @param proof The serialized ZK proof
    function submitBalanceProof(
        bytes32 vaultId,
        uint256 minBalance,
        uint256 blockHeight,
        bytes calldata proof
    ) external;

    /// @notice Get the latest attested balance for a vault
    function getAttestedBalance(bytes32 vaultId)
        external view returns (uint256 balance, uint256 blockHeight);
}
```

### 8.3 Fallback: M-of-N Attestor Model

Until PCD proof verification on Ethereum is production-ready, OGBank supports an M-of-N attestor model as a fallback. N independent attestors each hold the vault's FVK and independently scan the chain. They submit signed balance attestations to the oracle contract. This requires `M ≥ N/2 + 1` honest attestors.

The FVK is shared ONLY with attestors in this fallback model, NOT with the Relayer.

---

## 9. Vault Lifecycle

### 9.1 State Machine

```
INACTIVE ──[deposit]──> ACTIVE ──[ticket batch created]──> DELEGATED
    ↑                      ↑                                   │
    │                      │                                   │
    │                      └───────[spend executed]─────────────┘
    │                      ↑                                   │
    │                      │                                   │
    │                      └───[new ticket batch]──── RENEWAL ←┘
    │                                                          │
    │                      ┌───[revoke / tickets expire]───────┘
    │                      ↓
    │                   FROZEN ──[owner sweep (shares 1+2)]──> SWEPT
    │                                                           │
    └───────────────────────────────────────────────────────────┘
```

**INACTIVE**: Vault created, no funds deposited.
**ACTIVE**: Vault funded, no active delegation.
**DELEGATED**: Active tickets exist; Relayer can initiate spends.
**RENEWAL**: Tickets running low; Owner creates new batch.
**FROZEN**: All tickets expired or revoked; Relayer cannot spend.
**SWEPT**: Owner moved all funds out using shares 1+2. Vault is empty.

### 9.2 Revocation

The Owner revokes delegation by:
1. Not creating new ticket batches (passive revocation — tickets expire naturally)
2. Instructing the Signer to reject all future signing requests (active revocation)
3. Sweeping the vault using Owner A + Owner B shares (emergency revocation)

The Relayer cannot prevent revocation because the Owner holds 2-of-3 shares.

### 9.3 Share Refresh

FROST supports share refresh: rotating the secret shares without changing the group public key `ak`. This allows the Owner to revoke the Relayer's share and issue a new one to a different Relayer, without changing the vault address or moving funds.

The refresh ceremony uses `frost::keys::refresh` with either the Trusted Dealer or DKG variant. After refresh, the old shares are invalidated and the new shares are distributed.

---

## 10. Communication Protocol

### 10.1 Transport

All inter-participant communication MUST be authenticated and encrypted. Recommended transports in order of privacy preference:

1. **Nym mixnet**: Highest metadata privacy. Suitable for Signer ↔ Relayer communication where latency tolerance is >2s.
2. **Noise Protocol Framework (IK pattern)**: Low latency, both parties know each other's static keys. Suitable for Signer ↔ Relayer direct connections.
3. **Tor hidden service**: Good metadata privacy. Suitable for Relayer exposing an endpoint.
4. **TLS 1.3**: Minimum acceptable. Suitable for initial setup and Owner ↔ Signer local communication.

### 10.2 Message Types

```
enum OGBankMessage {
    // Key ceremony (§4.3)
    DKGRound1Package(frost::dkg::round1::Package),
    DKGRound2Package(frost::dkg::round2::Package),
    DKGComplete { pubkey_package, vault_address },

    // Ticket management (§5.6)
    TicketBatchForRelayer(RelayerTicketBatch),
    TicketBatchForSigner(SignerTicketBatch),

    // Chain sync (§6)
    NullifierWatchRequest(WatchRequest),
    NullifierAlert(NullifierAlert),
    NoteReceived(EncryptedNoteDetails),

    // Signing (§7.2–7.4)
    SpendableNoteRequest { vault_id, required_amount },
    SpendableNoteResponse(SpendableNoteSet),
    UnsignedTxForProof(UnsignedTransaction),
    ProofAndSighash { proof, rk, sighash, randomizer },
    SigningRequest(SigningRequest),
    SignatureShareResponse(Result<SignatureShare, SignerError>),

    // Lifecycle (§9)
    RevocationNotice { vault_id, owner_signature },
    ShareRefreshRound1(frost::keys::refresh::Package),
}
```

### 10.3 Message Serialization

All messages are serialized using a compact binary encoding. Multi-byte integers are little-endian (consistent with ZCash conventions). Variable-length fields are prefixed with a 2-byte length. Messages are framed with:

```
[message_type: u8][payload_length: u32_le][payload: variable][hmac: 32 bytes]
```

The HMAC is computed over `message_type || payload_length || payload` using a session key derived during transport handshake.

---

## 11. Security Considerations

### 11.1 Nonce Safety

FROST nonce reuse enables complete key recovery. OGBank mitigates this:

- Each ticket binds exactly one nonce pair. The Signer's `nonce_registry` maps `auth_nullifier_hash → SigningNonces` bijectively.
- Once a nonce is used (signature share produced), the entry is deleted from `nonce_registry` before returning the share.
- If the Signer crashes after producing a share but before deleting the nonce, it MUST check on restart whether the corresponding nullifier was spent on-chain. If spent, delete the nonce. If not, the nonce MAY be reused for the same ticket (same SIGHASH), but MUST NOT be used for a different message.
- The Relayer's nonce pool uses independent nonces generated from its own KeyPackage. These are also single-use.

### 11.2 Signer Availability

If the Signer goes offline, delegated spending stops. Funds remain safe (no one can spend with only the Relayer's 1-of-3 share). The Owner can still spend using shares 1+2 (Owner A + Owner B).

To improve availability, the Owner MAY run multiple Signer replicas with replicated state. However, state replication of the spent-nullifier SMT must be consistent to prevent double-use of tickets. Implementations SHOULD use a consensus mechanism between replicas or accept that only one replica is active at a time (leader election).

### 11.3 Ticket Exhaustion

If all tickets are consumed or expired and the Owner is unreachable, the Relayer cannot initiate new spends. This is by design — it bounds the Owner's exposure. The Owner SHOULD monitor ticket inventory and create new batches proactively.

### 11.4 Front-Running Protection

The Relayer's signing request includes the full SIGHASH, which commits to all transaction details. The Signer verifies SIGHASH consistency (Check 7 in §7.3). An attacker who intercepts a signing request cannot substitute a different transaction because the signature share is bound to the specific SIGHASH. This is analogous to Tornado Cash's circuit including `recipient`, `relayer`, and `fee` as public inputs to prevent front-running.

### 11.5 Privacy Leakage Summary

| Information | Who learns it |
|---|---|
| Vault address | Owner, Relayer, Signer (public) |
| Vault balance | Signer only (via FVK) |
| Transaction history | Signer only (via FVK) |
| Individual spend details | Relayer (only for spends it initiates) |
| Nullifier values (opaque) | NullifierWatch, Relayer (no linkage to notes) |
| Authorization ticket params | Relayer (knows tickets it holds) |
| FROST group public key | All participants + on-chain (as rk, re-randomized) |

---

## 12. Rust Crate Dependencies

```toml
[dependencies]
# FROST threshold signatures (Zcash Foundation)
frost-core = "3.0"
frost-rerandomized = "3.0"

# ZCash cryptographic primitives
reddsa = { version = "*", features = ["frost"] }
pasta_curves = "*"
orchard = "*"
zcash_primitives = "*"
zcash_client_backend = "*"
zip32 = "*"

# Hashing
blake2b_simd = "1.0"

# Serialization
serde = { version = "1.0", features = ["derive"] }
bincode = "1.0"

# Secure memory
zeroize = { version = "1.0", features = ["derive"] }

# Transport
snow = "0.9"              # Noise Protocol Framework
tokio = { version = "1.0", features = ["full"] }
tonic = "0.12"            # gRPC for lightwalletd

# Randomness
rand = "0.8"
```

---

## 13. Test Vectors

Test vectors SHALL be provided in a companion document covering:

1. Vault derivation from known seed + nonce → expected vault_id, address
2. Ticket generation from known auth_secret → expected commitment, nullifier_hash
3. Auth Tree construction from known commitments → expected root
4. FROST DKG from known random seeds → expected shares and group key
5. Signing ceremony from known nonces + SIGHASH → expected signature
6. Authorization validation: valid request → accept; each of 7 checks failing → specific rejection

---

## 14. References

| ID | Document |
|---|---|
| ZIP-32 | Shielded Hierarchical Deterministic Wallets |
| ZIP-224 | Orchard Shielded Protocol |
| ZIP-302 | Standardized Memo Field Format |
| ZIP-312 | FROST for Spend Authorization Multisignatures |
| RFC 9591 | Two-Round Threshold Schnorr Signatures with FROST |
| ePrint 2024/436 | Re-Randomized FROST (Gouvêa, Komlo) |
| Tachyon | Scaling Zcash with Oblivious Synchronization (Bowe, 2025) |
| Tornado Cash | Tornado Cash Privacy Solution v1.4 |

---

## Appendix A: Domain Separators

All BLAKE2b invocations use a 16-byte personalization parameter as domain separator:

| Domain Separator | Usage |
|---|---|
| `"OGBank__VaultIdx"` | Vault account index derivation (§4.1) |
| `"OGBank__VaultID_"` | Vault identifier computation (§4.3.3) |
| `"OGBank_AuthNull_"` | Auth nullifier from auth_secret (§5.1.2) |
| `"OGBank_AuthDest"` | Destination address hash (§5.1.2) |
| `"OGBank_NullHash"` | Auth nullifier hash (§5.1.3) |
| `"OGBank_AuthComm_"` | Authorization commitment (§5.2) |
| `"OGBank_TreeNode"` | Auth Merkle tree internal node (§5.3) |
| `"OGBank_TreeLeaf"` | Auth Merkle tree empty leaf (§5.3) |
| `"OGBank_SMTNode_"` | Spent nullifier SMT node (§5.5) |

---

## Appendix B: Wire Format Summary

All integers are little-endian. All hashes are 32 bytes. All Pallas points are 32 bytes (compressed). All Pallas scalars are 32 bytes.

```
AuthorizationTicket (wire):
  [auth_secret:      32 bytes]
  [max_amount:        8 bytes (u64 LE)]
  [destination_hash: 32 bytes]
  [expiry_block:      4 bytes (u32 LE)]
  [nonce_hiding:     32 bytes (compressed Pallas point)]
  [nonce_binding:    32 bytes (compressed Pallas point)]
  Total: 140 bytes

AuthorizationProof (wire):
  [auth_secret:          32 bytes]
  [auth_nullifier_hash:  32 bytes]
  [max_amount:            8 bytes]
  [destination_hash:     32 bytes]
  [expiry_block:          4 bytes]
  [merkle_proof_depth:    1 byte]
  [merkle_proof:         depth × 33 bytes (32 hash + 1 direction bit)]
  Total: 108 + depth × 33 bytes (at depth 20: 768 bytes)

SigningRequest (wire):
  [sighash:             32 bytes]
  [randomizer:          32 bytes (Pallas scalar)]
  [num_commitments:      2 bytes]
  [commitments:         num × 66 bytes (2 id + 32 hiding + 32 binding)]
  [auth_proof:          variable (see above)]
  [proposed_tx_hash:    32 bytes]
  [proposed_amount:      8 bytes]
  [proposed_dest_hash:  32 bytes]
```

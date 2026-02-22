# ZLend Kohaku Adapter — Feasibility Analysis

> Can Kohaku serve as the client-side infrastructure for ZLend? Should we build an adapter?

---

## 0. Why Kohaku — The Real Motivation

This isn't about storage convenience or ecosystem signaling. The motivation is **custody avoidance**.

### The Problem

ZLend's relayer generates a ZCash escrow address and gives the user a **viewing key (vk)**. This viewing key is the ONLY thing that lets the user:
1. See their ZEC balance in the escrow
2. Generate ZK proofs of deposit (needed to borrow)
3. Generate repayment proofs (needed to claim ZEC back)

**If the user loses the viewing key, they lose access to their deposit.** The ZEC is still in the escrow, but without the vk, the user cannot prove they own it.

### Why ZLend Can't Hold the Viewing Key

If ZLend stores viewing keys on behalf of users:
- ZLend becomes a **custodian of access credentials** — legal and operational liability
- A ZLend server breach leaks all viewing keys — attackers can scan every user's escrow balance
- Users must trust ZLend's infrastructure uptime and security — defeats the decentralization thesis
- The relayer already holds the spending key (custodial risk acknowledged). Holding BOTH keys makes ZLend a single point of total failure.

### Why Kohaku Solves This

Kohaku delegates key custody to the **user's wallet**. The viewing key lives in the wallet's `Host.storage` — managed by the wallet application, not by ZLend's servers.

This means:
- **ZLend never touches the viewing key** — the plugin receives it from the relayer and persists it in the wallet
- **The wallet is responsible for backup** — same as it handles mnemonics, private keys, etc.
- **ZLend can honestly say:** "Your viewing key is in your wallet. We don't store it. Back up your wallet."
- **If ZLend goes down**, users still have their vk in their wallet and can generate proofs independently

This is the difference between "ZLend manages your keys" and "your wallet manages your keys via the ZLend plugin."

---

## 1. Concept Mapping

How ZLend's primitives map (or don't) to Kohaku's abstractions:

### Direct Mappings

| ZLend Concept | Kohaku Equivalent | Fit |
|---------------|-------------------|-----|
| ZLend escrow account | `PluginInstance` | Direct — one instance per escrow account |
| Depositing ZEC | `prepareShield()` | Conceptual fit — public→private (user sends ZEC to escrow) |
| Claiming ZEC back | `prepareUnshield()` | Conceptual fit — private→public (escrow returns ZEC to user) |
| Viewing key storage | `Host.storage` | Direct — key-value persistence |
| Borrow nonce tracking | `Host.storage` | Direct — simple counter |
| Avalanche RPC calls | `Host.provider` | Direct — EthereumProvider for contract interaction |
| ZCash RPC calls | `Host.network` | Indirect — can use `fetch()` to call ZCash JSON-RPC |
| Relayer communication | `Broadcaster` or `Host.network` | Either works — broadcaster for structured API, network for raw HTTP |
| USDC balance queries | `PluginInstance.balance()` | Direct — query ERC-20 balances |

### Partial Mappings (Require Adaptation)

| ZLend Concept | Kohaku Closest | Gap |
|---------------|---------------|-----|
| ZCash viewing key (ZIP-32 Sapling) | `Host.keystore.deriveAt()` (BIP-32) | Different derivation path standard. ZLend's vk comes FROM the relayer, not from user's mnemonic. Keystore can't derive ZCash Sapling keys. |
| User secret (for nullifiers) | `Host.keystore.deriveAt()` | **Good fit here** — user_secret CAN be derived from keystore at a ZLend-specific path like `m/44'/zlend'/0'/0'/0` |
| Borrow nullifier | Railgun's `Note.getNullifier()` | Different formula. Railgun: `Poseidon(nullifyingKey, leafIndex)`. ZLend: `H(user_secret, borrow_nonce, state_root)`. Must implement custom. |
| ZK proof generation | Railgun's circuit system | Different proof system. Railgun: Groth16/snarkjs. ZLend: Ultrahonk/Noir/Barretenberg. Incompatible circuits. |

### No Mapping (ZLend-Specific)

| ZLend Concept | Why No Kohaku Equivalent |
|---------------|-------------------------|
| Cross-chain model (ZCash ↔ Avalanche) | Kohaku is single-chain (Ethereum/EVM). ZLend bridges two fundamentally different chains. |
| Relayer-held spending key | Kohaku assumes the user holds all keys. ZLend's escrow model is custodial on ZCash side. |
| Merkle tree of contract state roots | Not client-maintained. ZLend's state roots are on-chain (Avalanche), not client-indexed. |
| Private transfers between ZLend users | Not applicable — ZLend doesn't support private transfers, only borrow/repay cycles. |
| Aave V3 interaction | Protocol-specific — no Kohaku equivalent. Smart contract calls. |

---

## 2. Adapter Design

### 2.1 Type Definitions

```typescript
// Account identifier — the escrow's viewing key hash or Avalanche address
type ZLendAddress = `zlend:${string}`;

// Asset amounts — ZLend only handles ERC20 (USDC borrow/repay)
type ZLendAssetAmount = AssetAmount<ERC20AssetId>;

// Custom operation types
type ZLendPrivateOperation = PrivateOperation & {
    proof: Uint8Array;          // Ultrahonk proof
    nullifier: Uint8Array;      // Borrow nullifier
    amount: bigint;             // Borrow/claim amount
};

type ZLendPublicOperation = PublicOperation & {
    escrowAddress: string;      // ZCash escrow address
    amount: bigint;             // ZEC deposit amount
};

// Instance with supported features
type ZLendInstance = PluginInstance<
    ZLendAddress,
    {
        input: ZLendAssetAmount,
        internal: ZLendAssetAmount,
        output: ZLendAssetAmount,
    },
    ZLendPrivateOperation,
    {
        prepareShield: true,        // Deposit ZEC → escrow
        prepareUnshield: true,      // Claim ZEC back after repayment
        // NO transfer — ZLend doesn't support private transfers
        // NO multi-asset — ZLend handles one escrow at a time
    }
>;

// Broadcaster for relayer communication
type ZLendBroadcasterParams = {
    relayerUrl: string;
    // Relayer API configuration
};
type ZLendBroadcaster = Broadcaster<ZLendBroadcasterParams, ZLendPrivateOperation>;

// Plugin parameters
type ZLendPluginParams = {
    zlendContractAddress: string;   // ZLendContract on Avalanche
    relayerUrl: string;             // Relayer API endpoint
    zcashRpcUrl: string;            // ZCash node RPC
};

// Full plugin type
type ZLendPlugin = Plugin<
    "zlend",
    ZLendInstance,
    ZLendPrivateOperation,
    Host,
    ZLendBroadcaster,
    ZLendPluginParams
>;
```

### 2.2 Plugin Factory

```typescript
const createZLendPlugin: CreatePluginFn<ZLendPlugin> = async (host, params) => {
    const { zlendContractAddress, relayerUrl, zcashRpcUrl } = params;

    // Verify chain — ZLend requires Avalanche C-Chain
    const chainId = await host.provider.getChainId();
    if (chainId !== 43114n && chainId !== 43113n) {
        throw new UnsupportedChainError(chainId);
    }

    // Derive user secret from keystore (deterministic, reproducible)
    const userSecret = host.keystore.deriveAt("m/44'/7777'/0'/0'/0");

    // Load persisted state
    const state = loadZLendState(host.storage);

    // Create broadcaster for relayer communication
    const broadcaster: ZLendBroadcaster = {
        config: async (params) => { /* configure relayer URL */ },
        broadcast: async (operation) => {
            // Submit proof to relayer or directly to Avalanche
            await host.network.fetch(relayerUrl + '/submit', {
                method: 'POST',
                body: JSON.stringify(operation),
            });
        },
    };

    const instances: ZLendInstance[] = [];

    const createInstance = async (): Promise<ZLendInstance> => {
        // 1. Request account from relayer
        const { escrowAddress, viewingKey } = await requestAccount(
            host.network, relayerUrl, userSecret
        );

        // 2. Persist viewing key
        host.storage.set('zlend:vk', viewingKey);
        host.storage.set('zlend:escrow', escrowAddress);

        // 3. Create instance
        const instance: ZLendInstance = {
            instanceId: async () => `zlend:${escrowAddress}` as ZLendAddress,

            balance: async (assets) => {
                // Query ZCash escrow balance via viewing key
                const zecBalance = await scanEscrowBalance(
                    host.network, zcashRpcUrl, viewingKey, escrowAddress
                );
                // Query USDC balance on Avalanche
                const usdcBalance = await queryERC20Balance(
                    host.provider, zlendContractAddress
                );
                return [
                    { asset: { __type: 'erc20', contract: ZEC_PLACEHOLDER }, amount: zecBalance },
                    { asset: { __type: 'erc20', contract: USDC_ADDRESS }, amount: usdcBalance },
                ];
            },

            prepareShield: async (asset, to) => {
                // "Shield" = deposit ZEC to escrow
                // Returns instructions for the user to send ZEC
                return {
                    __type: 'publicOperation',
                    escrowAddress,
                    amount: asset.amount,
                } as ZLendPublicOperation;
            },

            prepareUnshield: async (asset, to) => {
                // "Unshield" = generate repayment proof + claim ZEC
                const nonce = getAndIncrementNonce(host.storage);
                const stateRoot = await getContractStateRoot(host.provider, zlendContractAddress);
                const nullifier = computeNullifier(userSecret, nonce, stateRoot);
                const proof = await generateRepaymentProof(/* ... */);

                return {
                    __type: 'privateOperation',
                    proof,
                    nullifier,
                    amount: asset.amount,
                } as ZLendPrivateOperation;
            },
        };

        instances.push(instance);
        return instance;
    };

    return {
        plugin_name: "zlend",
        createInstance,
        instances: () => instances,
        broadcaster,
    };
};
```

### 2.3 Storage Schema

What goes in `Host.storage` (key-value):

| Key | Value | Sensitivity | Purpose |
|-----|-------|-------------|---------|
| `zlend:vk` | Hex-encoded viewing key | HIGH | Escrow balance scanning + proof generation |
| `zlend:escrow` | ZCash escrow address | MEDIUM | Deposit target |
| `zlend:nonce` | Current borrow nonce (string number) | HIGH | Nullifier computation |
| `zlend:nullifiers` | JSON array of active nullifier hex strings | MEDIUM | Track active borrow cycles |
| `zlend:history` | JSON array of borrow/repay records | LOW | UI display |
| `zlend:lastBlock` | Last scanned Avalanche block number | LOW | Event polling resume |

What goes in `Host.keystore`:

| Path | Purpose |
|------|---------|
| `m/44'/7777'/0'/0'/0` | User secret derivation (for nullifier computation) |

**Note on path choice:** `7777` is an unused SLIP-44 coin type. Using a unique path ensures ZLend's user_secret doesn't collide with any other protocol's key derivation.

---

## 3. Comparison with Existing Adapters

| Aspect | Railgun | Privacy Pools v1 | Tornado | **ZLend (proposed)** |
|--------|---------|-------------------|---------|----------------------|
| **Account ID** | `0zk${string}` | Ethereum address | Derivation index | `zlend:${escrowAddr}` |
| **Key model** | Client derives spending+viewing | TBD | TBD | Relayer holds sk, client holds vk |
| **Shield** | ERC20 → private UTXO | ERC20 → private | ETH/ERC20 → commitment | ZEC → escrow (cross-chain) |
| **Transfer** | Private → private | v2 only | No | No |
| **Unshield** | Private UTXO → ERC20 | Yes | Yes (withdraw) | Repay proof → claim ZEC |
| **Merkle tree** | Client-maintained (Poseidon) | TBD | TBD | Not needed (on-chain state) |
| **Proof system** | Groth16 (snarkjs/circom) | TBD | TBD | Ultrahonk (Noir/Barretenberg) |
| **Indexer** | Full (events → tree) | TBD | TBD | Minimal (event polling only) |
| **Broadcaster** | No | Yes (URL) | Yes (URL) | Yes (relayer API) |
| **Cross-chain** | No (single EVM chain) | No | No | **Yes (ZCash ↔ Avalanche)** |
| **Complexity** | ~46K lines | ~200 lines (stub) | ~100 lines | ~500-1000 lines estimated |

### Key Differences from Railgun

1. **No client-side Merkle tree.** ZLend doesn't maintain a commitment tree in the browser. State roots come from the Avalanche contract. This eliminates Railgun's most complex component (~5K lines of Merkle tree + indexer code).

2. **No note encryption/decryption.** ZLend doesn't transfer private notes between users. The viewing key is used for ZCash balance scanning, not for decrypting commitment ciphertexts.

3. **Cross-chain by design.** Railgun operates on a single EVM chain. ZLend bridges ZCash (non-EVM, UTXO model) and Avalanche (EVM). The `Host.network.fetch()` must reach both a ZCash RPC and an Avalanche RPC.

4. **Different proof system.** Railgun uses Groth16 (snarkjs). ZLend uses Ultrahonk (Noir/Barretenberg). Circuit artifacts, proving, and verification are incompatible.

5. **Custodial escrow.** Railgun is fully non-custodial (user holds spending key). ZLend's relayer holds the spending key. This simplifies the adapter (no spending key management) but adds trust assumptions.

---

## 4. What Fits Well

### Host.storage — Viewing Key + State Persistence

Kohaku's key-value storage is a natural fit for ZLend's client state. The viewing key, borrow nonce, active nullifiers, and history all serialize cleanly to JSON strings.

```typescript
// Example: Persist viewing key
host.storage.set('zlend:vk', viewingKeyHex);

// Example: Track borrow nonce
const nonce = parseInt(host.storage.get('zlend:nonce') ?? '0');
host.storage.set('zlend:nonce', (nonce + 1).toString());

// Example: Track active nullifiers
const nullifiers = JSON.parse(host.storage.get('zlend:nullifiers') ?? '[]');
nullifiers.push(newNullifierHex);
host.storage.set('zlend:nullifiers', JSON.stringify(nullifiers));
```

### Host.keystore — User Secret Derivation

The keystore's `deriveAt(path)` can deterministically derive the `user_secret` used in nullifier computation. This means:
- The user_secret is reproducible across sessions (same mnemonic → same secret)
- No need to store the user_secret directly — it's re-derived from the wallet's mnemonic
- Follows the same pattern Railgun uses for spending/viewing key derivation

```typescript
const userSecret = host.keystore.deriveAt("m/44'/7777'/0'/0'/0");
// This is deterministic — same mnemonic always produces same secret
```

### Host.provider — Avalanche Contract Interaction

The `EthereumProvider` interface covers all of ZLend's Avalanche needs:
- `getLogs()` for scanning ZLendContract events (FinishPayment, etc.)
- `getBlockNumber()` for event polling resume
- Transaction submission for borrow/repay/claim

Avalanche C-Chain is EVM-compatible, so the existing provider adapters (ethers, viem) work without modification.

### Host.network — ZCash RPC + Relayer API

The `fetch()` interface can reach:
- ZCash JSON-RPC for UTXO scanning via viewing key
- ZLend relayer API for account creation and ZEC return signaling

### Broadcaster — Relayer Communication

The `Broadcaster` pattern maps well to ZLend's relayer:
- `config({ relayerUrl })` — Set relayer endpoint
- `broadcast(operation)` — Submit proof + nullifier to relayer for on-chain execution

### Plugin Pattern — Account Lifecycle

The `createInstance()` / `instances()` pattern maps to ZLend accounts:
- Each instance = one escrow account
- `instanceId()` = escrow address identifier
- `balance()` = cross-chain balance (ZEC in escrow + USDC on Avalanche)

---

## 5. What Doesn't Fit

### 5.1 ZCash Key Derivation ≠ BIP-32

**Problem:** Kohaku's `Host.keystore.deriveAt()` uses BIP-32 (secp256k1 curve). ZCash Sapling uses ZIP-32 (Jubjub curve, BLAKE2b-512 PRF). They are cryptographically incompatible.

**Impact:** The viewing key CANNOT be derived from `Host.keystore`. It must come from the relayer.

**Mitigation:** This is already how ZLend works — the relayer provides the viewing key. We just store it in `Host.storage` instead of deriving it from `Host.keystore`.

### 5.2 No SecretStorage in Host

**Problem:** `Host.storage` is explicitly plaintext. The viewing key and user secret are sensitive data. The `SecretStorage` type is defined in Kohaku but NOT included in the `Host` interface.

**Impact:** Sensitive ZLend data (viewing key, user secret) would be stored in plaintext unless the host application provides encryption externally.

**Mitigation options:**
1. Store viewing key encrypted with a user-provided password in `Host.storage`
2. Derive user_secret from `Host.keystore` (which IS secure — the host manages mnemonic security)
3. Propose adding `SecretStorage` to the `Host` interface (Kohaku contribution)

### 5.3 No Cross-Chain Provider Abstraction

**Problem:** `Host.provider` is an `EthereumProvider`. ZLend needs both an Avalanche provider AND a ZCash RPC client. They have different interfaces.

**Impact:** ZCash RPC calls must go through `Host.network.fetch()` as raw HTTP requests, not through the typed `EthereumProvider`.

**Mitigation:** Use `Host.network.fetch()` for ZCash JSON-RPC. This works but loses type safety. Could create a ZCash RPC wrapper that uses fetch internally.

### 5.4 Proof System Incompatibility

**Problem:** Railgun uses Groth16/snarkjs. ZLend uses Ultrahonk/Noir/Barretenberg. The circuit system (`circuits/index.ts`) is tightly coupled to Groth16.

**Impact:** Cannot reuse Railgun's circuit fetching, proving, or verification code.

**Mitigation:** ZLend adapter brings its own proof generation using `@aztec/bb.js` (Barretenberg WASM) or `@noir-lang/noir_js`. The circuit artifacts are fetched via `Host.network.fetch()`.

### 5.5 Plugin Instance Semantics Mismatch

**Problem:** `prepareShield()` in Kohaku means "deposit ERC-20 into the privacy system." In ZLend, the "shield" equivalent is "send ZEC from your ZCash wallet to the escrow address" — which is a cross-chain action the plugin can't execute. The plugin can only return instructions.

**Impact:** `prepareShield()` returns an address + instructions, not an executable transaction. The user must manually send ZEC from their ZCash wallet.

**Mitigation:** Return a `PublicOperation` with the escrow address and amount. The frontend interprets this as "show the user a QR code to send ZEC." This is a semantic stretch but works within the type system.

---

## 6. Gaps and Open Questions

| Gap | Severity | Notes |
|-----|----------|-------|
| No `SecretStorage` in Host interface | Medium | Sensitive data in plaintext. User secret can be derived from keystore (mitigated), but viewing key has no secure storage path. |
| No ZCash provider type | Low | Can use `Host.network.fetch()` as workaround. Type safety lost. |
| Ultrahonk circuits not in Kohaku | Low | Bring our own — no expectation of reuse. |
| `prepareShield` semantic mismatch | Low | Returns instructions, not a tx. Frontend must interpret. |
| Multi-chain event polling | Medium | Need to poll both Avalanche (via provider) and ZCash (via network.fetch). No built-in multi-chain indexer. |
| Account recovery from mnemonic | Open | Can re-derive user_secret from keystore, but viewing key must be re-fetched from relayer. If relayer is down, account data is partially lost. |
| Nonce synchronization | Open | Borrow nonce must stay in sync between client and contract. If client state is lost, nonce must be recovered from on-chain events. |

---

## 7. Recommendation

### Build the adapter. The viewing key custody argument is decisive.

**The core argument:**

ZLend's relayer already holds the spending key (acknowledged custodial risk). If ZLend ALSO holds users' viewing keys — either on servers or in ZLend-managed browser storage — then ZLend becomes a single point of total failure. A breach or shutdown means users lose both access AND funds.

Kohaku solves this by putting the viewing key in the user's wallet. ZLend never touches it after the initial handoff from the relayer. The wallet handles backup, encryption, and persistence — the same way it handles mnemonics and private keys.

**This is not optional infrastructure.** It's the architectural decision that lets ZLend say: "We hold your ZEC in escrow during the loan. But the key to prove your ownership? That's in your wallet. Not on our servers."

**Additional benefits:**

1. **Standardized wallet integration.** Any wallet that supports Kohaku plugins gets ZLend for free.

2. **Infrastructure reuse.** Provider abstraction, storage, key derivation — already built.

3. **Ecosystem signal.** ZLend as a Kohaku adapter = alignment with EF's privacy vision. Credibility for partnerships (Avalanche Foundation, ZCash Foundation, Aztec).

4. **Low implementation cost.** ~800 lines vs Railgun's 46K. No Merkle trees, no note encryption.

5. **Account recovery path.** User secret is derivable from wallet mnemonic (`Host.keystore`). If viewing key is lost but wallet mnemonic is backed up, user can re-request vk from relayer by proving identity via user_secret.

**Risks:**

1. **Kohaku is early.** Parts are WIP. SDK may change. But the core interfaces (`Plugin`, `PluginInstance`, `Host`) are stable and well-typed.

2. **Semantic friction.** `prepareShield` as "return deposit instructions" is a stretch. Acceptable — the type system allows it.

3. **No SecretStorage in Host.** Viewing key stored in plaintext `Host.storage`. Mitigated by: (a) wallet apps typically encrypt their storage, (b) user_secret derived from keystore is already secure, (c) can propose `SecretStorage` addition to Kohaku.

4. **Dependency on Kohaku adoption.** If no wallet adopts Kohaku, the adapter has no distribution. But the fallback is trivial — the same state can be managed with a 100-line localStorage wrapper for a standalone web app. The adapter doesn't lock us in.

### Implementation Priority

| Phase | What | Lines (est.) |
|-------|------|-------------|
| **Phase 1** | Type definitions + plugin factory + storage schema | ~200 lines |
| **Phase 2** | Balance queries (ZCash scan + Avalanche ERC-20) | ~150 lines |
| **Phase 3** | Proof generation integration (Ultrahonk/Noir) | ~200 lines |
| **Phase 4** | Broadcaster (relayer API communication) | ~100 lines |
| **Phase 5** | Event polling + nonce recovery | ~150 lines |

**Total: ~800 lines** — compare to Railgun's 46K. ZLend's adapter is an order of magnitude simpler because it doesn't need client-side Merkle trees, note encryption, or multi-party transfers.

### New Package Structure

```
kohaku/packages/zlend/
├── src/
│   ├── index.ts              # Re-exports
│   ├── plugin.ts             # createZLendPlugin factory
│   ├── instance.ts           # ZLendInstance type + createInstance
│   ├── broadcaster.ts        # Relayer API broadcaster
│   ├── storage.ts            # State persistence schema
│   ├── zcash/
│   │   ├── scanner.ts        # UTXO scanning via viewing key
│   │   └── rpc.ts            # ZCash JSON-RPC client
│   ├── proofs/
│   │   ├── borrow.ts         # Borrow proof generation
│   │   └── repayment.ts      # Repayment proof generation
│   └── config/
│       └── avalanche.ts      # ZLend contract addresses
├── package.json
└── tsconfig.json
```

---

## Links

- Kohaku codebase analysis: [01_kohaku-codebase.md](01_kohaku-codebase.md)
- ZLend data requirements: [02_zlend-data-requirements.md](02_zlend-data-requirements.md)
- ZLend protocol specification: [../02_protocol.md](../02_protocol.md)
- Kohaku research (existing): [../06_research.md](../06_research.md#4-kohaku-investigation)

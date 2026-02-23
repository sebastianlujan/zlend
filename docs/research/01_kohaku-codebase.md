# Kohaku Codebase Analysis

> Source: [ethereum/kohaku](https://github.com/ethereum/kohaku) — Ethereum Foundation's privacy wallet SDK.
> Codebase analyzed: February 2026. Local copy at `/kohaku`.

---

## Overview

Kohaku is the Ethereum Foundation's modular privacy framework. It provides a plugin architecture where different privacy protocols (Railgun, Privacy Pools, etc.) integrate behind a common interface. The SDK handles storage, key derivation, blockchain provider abstraction, and event processing.

**Core idea:** A wallet application implements the `Host` interface (providing storage, keys, RPC access), and Kohaku plugins provide privacy-preserving operations on top.

---

## Project Structure

```
kohaku/
├── packages/
│   ├── plugins/          @kohaku-eth/plugins      ✅ Ready
│   ├── provider/         @kohaku-eth/provider     ✅ Ready
│   ├── railgun/          @kohaku-eth/railgun      ✅ Ready
│   ├── privacy-pools/    @kohaku-eth/privacy-pools 🚧 WIP
│   ├── pq-account/       Post-quantum 4337 account ✅ Ready
│   └── docs/             Documentation (Vocs)
├── pnpm-workspace.yaml
└── .gitmodules
```

Monorepo managed with pnpm workspaces. TypeScript throughout (94.1%), with Solidity for smart contracts (5.7%).

---

## 1. Plugins Package (`@kohaku-eth/plugins`)

The abstraction layer that defines the interface every privacy protocol must implement. This is the most important package for understanding how adapters work.

### 1.1 Plugin Type

```
Source: packages/plugins/src/base.ts
```

```typescript
type Plugin<TName, TInstance, TPrivateOperation, THost, TBroadcaster, TExtraParams> = {
    plugin_name: TName;
    createInstance: () => Promise<TInstance> | TInstance;
    instances: () => Promise<TInstance[]> | TInstance[];
} & (TBroadcaster extends never ? {} : { broadcaster: TBroadcaster });
```

A plugin has a name, can create instances (accounts), list existing instances, and optionally has a broadcaster for submitting transactions.

**Factory pattern:**
```typescript
type CreatePluginFn<TPlugin> = (host: Host, params: PluginParams<TPlugin>) => Promise<TPlugin> | TPlugin;
```

Every plugin is created with a `Host` (the wallet environment) and plugin-specific parameters.

### 1.2 PluginInstance (Account Interface)

```
Source: packages/plugins/src/instance/base.ts
```

```typescript
type PluginInstance<TAccountId, TAssetAmounts, TPrivateOperation, TransactionFeatures> = {
    instanceId: () => Promise<TAccountId>;
    balance: (assets: AssetId[] | undefined) => Promise<AssetAmount[]>;
} & Transact<...>;  // Enabled transaction methods based on TransactionFeatures flags
```

Every instance (account) must provide:
- `instanceId()` — A unique account identifier (Railgun uses `0zk...`, Privacy Pools uses Ethereum addresses)
- `balance(assets?)` — Query balances for specified assets

Plus **optional transaction methods**, enabled via TypeScript flags:

| Method | Purpose |
|--------|---------|
| `prepareShield(asset, to?)` | Deposit from public → private (returns `PublicOperation`) |
| `prepareShieldMulti(assets, to?)` | Multi-asset deposit |
| `prepareTransfer(asset, to)` | Private → private transfer (returns `PrivateOperation`) |
| `prepareTransferMulti(assets, to)` | Multi-asset private transfer |
| `prepareUnshield(asset, to)` | Withdraw from private → public (returns `PrivateOperation`) |
| `prepareUnshieldMulti(assets, to)` | Multi-asset withdrawal |

The `TransactionFeatures` generic is a boolean flag object. Only methods with `true` flags exist on the instance type. This is compile-time enforced.

### 1.3 Host Interface

```
Source: packages/plugins/src/host/index.ts
```

The Host is what the wallet application provides to plugins:

```typescript
type Host = {
    network: Network;       // HTTP access (fetch)
    storage: Storage;       // Plaintext key-value persistence
    keystore: Keystore;     // BIP-32 key derivation
    provider: EthereumProvider;  // Blockchain RPC
};
```

#### Storage

```typescript
type Storage = {
    set(key: string, value: string): void;
    get(key: string): string | null;
};
```

Plaintext key-value store. Plugins MUST assume data is stored unencrypted. The host is responsible for any encryption at the storage layer level.

There's also a `SecretStorage` interface (encrypted at rest), but it's defined as a separate type — not yet integrated into the Host.

#### Keystore

```typescript
type Keystore = {
    deriveAt(path: string): Hex;  // BIP-32 path → private key
};
```

Derives private keys at a BIP-32 path. The host controls which paths are allowed. Same path always returns the same key.

#### Network

```typescript
type Network = {
    fetch(input: RequestInfo | URL, init?: RequestInit): Promise<Response>;
};
```

HTTP access for fetching external data (circuit artifacts, broadcaster APIs, etc.).

### 1.4 Asset Types

```
Source: packages/plugins/src/shared.ts
```

```typescript
type ERC20AssetId = { __type: 'erc20', contract: Address };
type ERC721AssetId = { __type: 'erc721', contract: Address, tokenId: bigint };
type AssetId = ERC20AssetId | ERC721AssetId;
type AssetAmount<TAssetId = AssetId, TAmount = bigint> = { asset: TAssetId, amount: TAmount };
```

### 1.5 Operations

```typescript
type PrivateOperation = Kind<"privateOperation">;  // For transfers/unshields
type PublicOperation = Kind<"publicOperation">;     // For shields
```

These are discriminated unions that adapters extend with their own data.

### 1.6 Broadcaster

```
Source: packages/plugins/src/broadcaster/base.ts
```

```typescript
type Broadcaster<TParameters, TPrivateOperation> = {
    config: (params: TParameters) => Promise<void>;
    broadcast: (operation: TPrivateOperation) => Promise<void>;
};
```

Optional transaction submission interface. Not all plugins need one (Railgun doesn't use it — transactions go directly via the provider).

### 1.7 Error Types

```
Source: packages/plugins/src/errors.ts
```

Standard error classes for plugin implementations:
- `UnsupportedAssetError`, `UnsupportedChainError`, `UnsupportedAccountError`
- `InvalidAddressError`, `InsufficientBalanceError`
- `MultiAssetsNotSupportedError`, `TransferNotSupportedError`

---

## 2. Provider Package (`@kohaku-eth/provider`)

Abstracts blockchain RPC providers behind a common interface.

```
Source: packages/provider/src/provider.ts
```

```typescript
type EthereumProvider<T> = {
    _internal: T;                                              // Raw provider
    getChainId(): Promise<bigint>;
    getLogs(params: Filter): Promise<TxLog[]>;
    getBlockNumber(): Promise<bigint>;
    waitForTransaction(txHash: string): Promise<void>;
    getBalance(address: string): Promise<bigint>;
    getCode(address: string): Promise<string>;
    getTransactionReceipt(txHash: string): Promise<TransactionReceipt | null>;
};
```

### Supported Providers

| Provider | File | Status |
|----------|------|--------|
| ethers v6 | `src/ethers/index.ts` | Complete |
| viem v2 | `src/viem/index.ts` | Complete |
| Colibri (stateless) | `src/colibri/index.ts` | Complete |
| Helios (a16z light client) | `src/helios/index.ts` | Complete |
| Raw JSON-RPC | `src/raw/index.ts` | Complete (base for Colibri/Helios) |

Each adapter maps provider-specific methods to the common interface. The `raw` adapter works directly with JSON-RPC calls and serves as the foundation for Colibri and Helios.

### Transaction Types

```
Source: packages/provider/src/tx.ts
```

```typescript
type TxData = { to: string, data: string, value: bigint };
type TxLog = { blockNumber: bigint, topics: string[], data: string, address: string };
type TransactionReceipt = { blockNumber: bigint, status: bigint, logs: TxLog[], gasUsed: bigint };
```

---

## 3. Railgun Package (`@kohaku-eth/railgun`)

The most complete adapter implementation. 46,391 lines of TypeScript. This is the reference for how a full Kohaku adapter works.

### 3.1 Architecture Overview

```
Plugin Factory (plugin.ts)
├── Indexer (indexer/base.ts)
│   ├── Event Processing (indexer/events/)
│   │   ├── Shield events
│   │   ├── Transact events
│   │   └── Nullified events
│   ├── Merkle Trees (railgun/logic/logic/merkletree.ts)
│   ├── RPC Sync (indexer/sync.ts)
│   └── Indexer Storage (indexer/storage.ts)
│
├── Account (account/base.ts)
│   ├── Key Derivation (account/keys.ts)
│   ├── Address Generation (account/actions/address.ts)
│   ├── Balance Queries (account/actions/balance.ts)
│   ├── Note Management (account/actions/notes.ts)
│   ├── Account Storage (account/storage.ts)
│   └── Transactions
│       ├── Shield (account/tx/shield.ts)
│       ├── Transfer (account/tx/transfer.ts)
│       └── Unshield (account/tx/unshield.ts)
│
├── Logic Layer
│   ├── Note (railgun/logic/logic/note.ts)
│   ├── Wallet (railgun/logic/logic/wallet.ts)
│   ├── Transaction Builder (railgun/logic/logic/transaction.ts)
│   └── Cryptography (railgun/logic/global/crypto.ts)
│
├── Circuits (circuits/index.ts)
│   └── HTTP Fetcher (circuits/fetchers/http.ts)
│
├── Config (config/)
│   ├── Mainnet
│   └── Sepolia
│
└── Storage Layer (storage/)
    ├── Base (storage/base.ts)
    ├── Empty Layer (storage/layers/empty.ts)
    └── File Layer (storage/layers/file.ts)
```

### 3.2 Key Derivation

```
Source: packages/railgun/src/account/keys.ts
```

Two ways to create keys:

**From mnemonic (BIP-39):**
```typescript
type KeyConfigMnemonic = {
    type: 'mnemonic';
    mnemonic: string;       // 12-word seed
    accountIndex: number;   // BIP-32 account index
};
```

**From private keys (direct import):**
```typescript
type KeyConfigPrivateKey = {
    type: 'key';
    spendingKey: string;    // Hex
    viewingKey: string;     // Hex
    ethKey?: string;        // Optional Ethereum signer
};
```

**Derived output:**
```typescript
type DerivedKeys = {
    spending: WalletNode;   // EdDSA (Baby Jubjub) — signs transactions
    viewing: WalletNode;    // EdDSA (Ed25519) — decrypts received notes
    master: bigint;         // Master public key (derived from spending + viewing)
    signer?: Wallet;        // Optional ethers Wallet for Ethereum interactions
};
```

The master public key is: `Poseidon(spendingPubKey.x, spendingPubKey.y, nullifyingKey)` where `nullifyingKey = Poseidon(viewingKey)`.

### 3.3 Note (Commitment) Model

```
Source: packages/railgun/src/railgun/logic/logic/note.ts
```

A Note is Railgun's equivalent of a UTXO:

```typescript
class Note {
    spendingKey: Uint8Array;    // 32 bytes — who can spend this
    viewingKey: Uint8Array;     // 32 bytes — who can see this
    value: bigint;              // Amount (max 2^128 - 1)
    random: Uint8Array;         // 16 bytes — uniqueness
    tokenData: TokenData;       // Asset type + address
    memo: string;               // User memo
}
```

**Key operations:**

| Method | Formula | Purpose |
|--------|---------|---------|
| `getNullifyingKey()` | `Poseidon(viewingKey)` | Derives the nullifier key |
| `getMasterPublicKey()` | `Poseidon(spendPubKey.x, spendPubKey.y, nullifyingKey)` | Account identifier |
| `getNotePublicKey()` | `Poseidon(masterPubKey, random)` | Unique per note |
| `getHash()` | `Poseidon(npk, tokenID, value)` | Merkle leaf commitment |
| `getNullifier(leafIndex)` | `Poseidon(nullifyingKey, leafIndex)` | Double-spend prevention |
| `sign(root, params, nullifiers, commitments)` | EdDSA over `Poseidon(sighash)` | Transaction authorization |

**Encryption:** Notes are encrypted for transfer using ECDH key exchange between sender/receiver viewing keys, then AES-GCM for the payload.

### 3.4 Merkle Tree

```
Source: packages/railgun/src/railgun/logic/logic/merkletree.ts
```

```typescript
class MerkleTree {
    treeNumber: number;         // Tree index (0, 1, 2...)
    depth: number;              // 16 (fixed)
    zeros: Uint8Array[];        // Precomputed zero hashes per level
    tree: Uint8Array[][];       // Sparse 2D array [level][index]
    nullifiers: Uint8Array[];   // All seen nullifiers for this tree
    maxLeafIndex: number;       // Highest populated leaf
}
```

Properties:
- **Depth 16** = 65,536 leaves per tree (constant `TOTAL_LEAVES = 2^16`)
- **Poseidon hash** for all internal nodes
- **Sparse storage** — only populated positions are stored
- **Multi-tree chaining** — when a tree fills, overflow goes to tree N+1
- **Lazy rebuild** — upper levels computed on demand via `rebuildSparseTree()`

Key methods:
- `insertLeaves(leaves, position)` — Add commitment hashes
- `get root()` — Current Merkle root
- `generateProof(element)` — Merkle inclusion proof

### 3.5 Indexer

```
Source: packages/railgun/src/indexer/base.ts
```

The indexer processes blockchain events and maintains Merkle tree state.

```typescript
type Indexer = {
    getTrees(): (MerkleTree | undefined)[];
    accounts: RailgunAccount[];
    registerAccount(account): void;
    processLogs(logs: TxLog[]): Promise<void>;
    getEndBlock(): number;
    sync?: (params?) => Promise<void>;  // RPC sync if provider available
};
```

**Event types processed:**

| Event | Source | Action |
|-------|--------|--------|
| `Shield` | Public deposit | Hash commitment preimages → insert as Merkle leaves, decrypt for registered accounts |
| `Transact` | Private transfer | Insert pre-hashed commitments as Merkle leaves, attempt decryption for accounts |
| `Nullified` | Note spent | Append nullifiers to tree's nullifier list (marks notes as spent) |

**RPC Sync** (`indexer/sync.ts`):
- Adaptive batch sizing: starts at 200 blocks, shrinks on RPC errors, grows on success
- 400ms pacing between RPC calls
- Periodic saves every 1000 blocks
- Handles reorgs and single-block failures

### 3.6 Account

```
Source: packages/railgun/src/account/base.ts
```

A `RailgunAccount` composes all the pieces:

```typescript
type RailgunAccount = {
    getRailgunAddress: () => Promise<'0zk...'>;
    getMasterPublicKey: () => Promise<bigint>;
    getBalance: (token?) => Promise<bigint>;
    getAllNotes: (treeIndex) => Note[];
    getUnspentNotes: (token) => Promise<Note[][]>;
    shield: (token, value) => Promise<TxData>;
    shieldNative: (value) => Promise<TxData>;
    transfer: (token, value, receiver) => Promise<TxData>;
    unshield: (token, value, recipient) => Promise<TxData>;
    indexer: Indexer;
    getSerializedState: () => CachedAccountStorage;
};
```

**Account Storage:**

```typescript
type AccountStorage = {
    notebooks: Notebook[];  // One per Merkle tree
    endBlock: number;       // Last processed block
};

type CachedAccountStorage = {
    notebooks: SerializedNoteData[][];  // Serialized notes
    endBlock: number;
};
```

Notes are organized in "notebooks" (sparse arrays matching Merkle tree leaf positions). When the indexer processes events, it checks if decrypted notes belong to each registered account and stores them in the appropriate notebook.

### 3.7 Transactions

**Shield** (`account/tx/shield.ts`):
1. Create ShieldNote with token, value, master public key
2. Sign a message with eth signer → derive ephemeral shield private key
3. Encrypt note random + receiver viewing key
4. Encode as ShieldRequest
5. Call `RailgunSmartWallet.shield([requests])`

**Transfer** (`account/tx/transfer.ts`):
1. Select unspent notes as inputs
2. Create output notes (receiver + change)
3. Generate ZK proof via `transact()` — includes Merkle root, nullifiers, new commitments, EdDSA signature
4. Call `RailgunSmartWallet.transact([publicInputs])`

**Unshield** (`account/tx/unshield.ts`):
- Same as transfer but output goes to an Ethereum address (0x...) instead of a Railgun address (0zk...)

### 3.8 Circuits

```
Source: packages/railgun/src/circuits/index.ts
```

```typescript
type Artifact = {
    zkey: Uint8Array;   // Proving key
    wasm: Uint8Array;   // Witness generator (WASM)
    vkey: VKey;         // Verification key
};

type ArtifactConfig = {
    nullifiers: number;     // Number of input notes
    commitments: number;    // Number of output notes
};
```

Circuits are parameterized by number of inputs/outputs. They're fetched via HTTP and cached with an LRU strategy. Proof system: Groth16 on BN128 curve.

### 3.9 Cryptographic Primitives

```
Source: packages/railgun/src/railgun/logic/global/crypto.ts
```

| Primitive | Library | Purpose |
|-----------|---------|---------|
| Poseidon hash | `circomlibjs` | ZK-friendly hashing (notes, nullifiers, Merkle tree) |
| EdDSA (Baby Jubjub) | Custom | Transaction signing (spending key) |
| Ed25519 | `@noble/ed25519` | Encryption key exchange (viewing key) |
| AES-GCM | `@noble/ciphers` | Note payload encryption |
| AES-CTR | `browserify-aes` | Sender-specific data encryption |
| SHA-256, SHA-512 | `@noble/hashes` | General hashing |
| Keccak-256 | `ethereum-cryptography` | EVM-compatible hashing |

### 3.10 Storage Layer

```
Source: packages/railgun/src/storage/base.ts
```

Three-level abstraction:

```
StorageLayer (raw read/write)
     ↓
StorageParser (serialize/deserialize)
     ↓
Storage<O> (typed load/save)
```

```typescript
type StorageLayer = {
    read: () => Promise<object | undefined>;
    write: (data: object) => Promise<void>;
};

type Storage<O> = {
    load: () => Promise<O>;
    save: (data: O) => Promise<void>;
};
```

**Implementations:**
- `EmptyStorageLayer` — In-memory (no persistence)
- `FileStorageLayer` — JSON file on disk (Node.js only, not browser)

The Railgun plugin bridges Host.storage (key-value) to StorageLayer:
```typescript
storage: {
    read: async () => JSON.parse(host.storage.get('indexer') ?? '{}'),
    write: async (data) => host.storage.set('indexer', JSON.stringify(data)),
}
```

### 3.11 Network Config

```
Source: packages/railgun/src/config/
```

```typescript
type RailgunNetworkConfig = {
    NAME: string;
    RAILGUN_ADDRESS: string;        // Smart contract address
    GLOBAL_START_BLOCK: number;
    CHAIN_ID: bigint;
    RELAY_ADAPT_ADDRESS: string;
    WETH: Address;
    FEE_BASIS_POINTS: bigint;       // Protocol fee (25 = 0.25%)
};
```

Supported: Ethereum mainnet (chainId 1), Sepolia testnet (chainId 11155111).

---

## 4. Privacy Pools Package (`@kohaku-eth/privacy-pools`)

Work-in-progress adapter for the Privacy Pools protocol (Buterin/Soleimani).

### v1

```
Source: packages/privacy-pools/src/v1/
```

```typescript
type PPv1Address = Address;  // Uses Ethereum addresses
type PPv1AssetAmount = AssetAmount<ERC20AssetId>;  // ERC20 only

type PPv1Instance = PluginInstance<PPv1Address, AssetAmounts, PPv1PrivateOperation, {
    prepareShield: true,
    prepareShieldMulti: true,
    prepareUnshield: true,
    prepareUnshieldMulti: true,
    // NO transfer support
}>;
```

Has a `Broadcaster` with `{ broadcasterUrl: string }` configuration. Implementation is currently a stub — all methods return empty objects.

### v2

Same pattern as v1 but adds `prepareTransfer` and `prepareTransferMulti` support. Also a stub.

---

## 5. Tornado Example

```
Source: packages/plugins/examples/tornado.ts
```

Minimal reference implementation showing:
- Fixed denomination pools (ETH: 0.1/1.0, DAI: 100/1000, USDC: 100/1000/10000)
- Account ID as derivation index (`'${number}'`)
- Only `prepareShield` + `prepareUnshield` (no transfers)
- Broadcaster with URL configuration
- Type-safe asset amounts (compile-time denomination checking)

---

## 6. PQ Account Package

Post-quantum ERC-4337 account abstraction. Solidity contracts built with Foundry. Not directly relevant to OGBank adapter but shows the breadth of Kohaku's scope.

---

## 7. Key Patterns for Adapter Development

### Pattern 1: Plugin Creation

```typescript
const createMyPlugin: CreatePluginFn<MyPlugin> = async (host, params) => {
    // 1. Get chain from provider
    const chainId = await host.provider.getChainId();

    // 2. Initialize state from storage
    const state = JSON.parse(host.storage.get('my-state') ?? '{}');

    // 3. Create instance pool
    const instances: MyInstance[] = [];

    return {
        plugin_name: "my-protocol",
        createInstance: async () => { /* ... */ },
        instances: () => instances,
    };
};
```

### Pattern 2: Storage Bridge

```typescript
// Host.storage (key-value) → StorageLayer (read/write)
const storageShim = {
    read: async () => JSON.parse(host.storage.get('key') ?? '{}'),
    write: async (data) => host.storage.set('key', JSON.stringify(data)),
};
```

### Pattern 3: Key Derivation

```typescript
// Use host keystore for protocol-specific keys
const myPrivateKey = host.keystore.deriveAt("m/44'/ogbank'/0'/0'/0");
```

### Pattern 4: Feature Flags

```typescript
// Declare supported operations at the type level
type MyInstance = PluginInstance<MyAddress, MyAssets, MyOperation, {
    prepareShield: true,       // Enabled
    prepareUnshield: true,     // Enabled
    // transfer: NOT listed = not available
}>;
```

---

## 8. TODOs and Incomplete Areas

| Area | Status | Note |
|------|--------|------|
| Railgun plugin instance methods | Stub | `balance`, `shield`, etc. return `undefined as never` |
| Railgun credential loading | TODO | "load from storage" not implemented |
| Privacy Pools v1/v2 | Stub | All methods return empty objects |
| SecretStorage integration | Type-only | Defined but not in Host interface |
| Legacy event scanning | TODO | "KASS TODO: also scan legacy events" |
| Relay Adapt V3 | TODO | ABI and verification hash pending |
| Circuit artifact hosting | Temporary | GitHub URL, pending stable release |

---

## Links

- GitHub: [ethereum/kohaku](https://github.com/ethereum/kohaku)
- OGBank data requirements: [02_ogbank-data-requirements.md](02_ogbank-data-requirements.md)
- Adapter feasibility: [03_ogbank-kohaku-adapter.md](03_ogbank-kohaku-adapter.md)


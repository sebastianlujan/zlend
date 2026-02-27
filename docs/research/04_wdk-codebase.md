# WDK Codebase Analysis

> Source: [@tetherto/wdk](https://github.com/nicola/wallet-sdk) — Tether's Wallet Development Kit.
> Codebase analyzed: February 2026. Local copy at `/wdk`.

---

## Overview

WDK is Tether's modular wallet orchestrator. It manages dynamic registration of wallet implementations for different blockchains and service protocols (swap, bridge, lending, fiat) through a single API. It acts as a central hub — it does not implement wallet logic itself but delegates to registered blockchain-specific wallet managers.

**Core idea:** You provide a BIP-39 seed phrase. WDK passes it to registered wallet implementations. Each wallet derives accounts from the seed. WDK decorates those accounts with protocol services.

---

## Project Structure

```
wdk/
├── src/
│   ├── wdk-manager.js           # Core manager class (~316 LOC)
│   └── wallet-account-with-protocols.js  # Account interface (~89 LOC)
├── types/
│   ├── index.d.ts               # Main type exports
│   ├── src/wdk-manager.d.ts     # Full WDK type definitions
│   └── src/wallet-account-with-protocols.d.ts  # Account interface types
├── tests/
│   └── wdk-manager.test.js      # 24 Jest test cases
├── .github/
│   └── workflows/
│       ├── build.yml            # Tests on push/PR
│       └── publish.yml          # npm publish on release
├── index.js                     # Entry point
├── bare.js                      # Bare Node Runtime support
└── package.json                 # v1.0.0-beta.5
```

TypeScript/JavaScript. ES modules. ~400 lines of implementation code. Apache 2.0 license.

---

## 1. Core Architecture: WdkManager

```
Source: src/wdk-manager.js
```

### Constructor

```javascript
constructor(seed) // seed: string (BIP-39 phrase) | Uint8Array (16-64 bytes)
```

Validates seed, stores it in `_seed`. Initializes:
- `_wallets` — Map of blockchain name → wallet manager instance
- `_protocols` — Nested object: `{ swap: {}, bridge: {}, lending: {}, fiat: {} }`
- `_middlewares` — Object of blockchain name → middleware function array

### Registration Pattern

Three registration methods, all return `this` for chaining:

**1. Wallet Registration**

```javascript
registerWallet(blockchain, WalletManager, config)
```

Creates `new WalletManager(seed, config)` and stores it keyed by blockchain name. The wallet manager owns the seed and all key derivation.

**2. Protocol Registration**

```javascript
registerProtocol(blockchain, label, Protocol, config)
```

Registers a service protocol (must be instanceof `SwapProtocol`, `BridgeProtocol`, `LendingProtocol`, or `FiatProtocol`). Stored as `{ Protocol, config }`. Label must be unique per type per blockchain.

**3. Middleware Registration**

```javascript
registerMiddleware(blockchain, middleware)
```

Registers an async function `(account) => Promise<void>` called when any account is derived for that blockchain. Multiple middlewares run sequentially.

### Account Derivation

```javascript
async getAccount(blockchain, index = 0)      // BIP-44 index-based
async getAccountByPath(blockchain, path)      // Custom BIP-44 path (e.g. "0'/0/0")
```

Flow:
1. Look up wallet manager for blockchain
2. Call `wallet.getAccount(index)` or `wallet.getAccountByPath(path)`
3. Run registered middlewares sequentially
4. Decorate account with protocol getter methods
5. Return `IWalletAccountWithProtocols`

### Other Methods

```javascript
async getFeeRates(blockchain)  // Delegates to wallet.getFeeRates()
dispose()                       // Calls dispose() on all wallets, clears map
static getRandomSeedPhrase()    // Returns random BIP-39 phrase
static isValidSeed(seed)        // Validates seed (string or 16-64 byte Uint8Array)
```

---

## 2. Account Interface (`IWalletAccountWithProtocols`)

```
Source: src/wallet-account-with-protocols.js
```

Extends `IWalletAccount` (from `@tetherto/wdk-wallet`) with protocol management:

```javascript
registerProtocol(label, Protocol, config)   // Per-account protocol registration
getSwapProtocol(label)                       // → ISwapProtocol
getBridgeProtocol(label)                     // → IBridgeProtocol
getLendingProtocol(label)                    // → ILendingProtocol
getFiatProtocol(label)                       // → IFiatProtocol
```

Protocol lookup: WDK-level first, then account-level.

### Base IWalletAccount Interface (inferred from usage)

```typescript
interface IWalletAccount {
    getAddress(): Promise<string>;
    sendTransaction(tx): Promise<{ hash, fee }>;
}

interface IWalletManager {
    constructor(seed: string | Uint8Array, config: object);
    getAccount(index: number): Promise<IWalletAccount>;
    getAccountByPath(path: string): Promise<IWalletAccount>;
    getFeeRates(): Promise<FeeRates>;
    dispose(): void;
}
```

---

## 3. Key Derivation Model

WDK does NOT derive keys. It passes the seed to wallet implementations and delegates.

```
BIP-39 Seed Phrase
       │
       ▼
   WDK Constructor (stores seed)
       │
       ├── registerWallet("ethereum", WalletManagerEvm, config)
       │         └── new WalletManagerEvm(seed, config)
       │                   └── BIP-44: m/44'/60'/0'/0/{index}
       │
       ├── registerWallet("bitcoin", WalletManagerBtc, config)
       │         └── new WalletManagerBtc(seed, config)
       │                   └── BIP-44: m/44'/0'/0'/0/{index}
       │
       └── registerWallet("zcash-viewer", CustomWallet, config)  ← hypothetical
                 └── ???
```

**Key constraints:**
- All accounts MUST be derived from the provided seed
- No mechanism to import external keys at the WDK level
- No key storage — keys are derived on demand, held in memory
- `dispose()` clears the seed from memory

---

## 4. Storage Model

**WDK provides zero storage.** No `Host.storage`, no keystore, no persistence layer, no encrypted storage.

| What | How |
|------|-----|
| Seed | In memory (`_seed`), erased on `dispose()` |
| Derived keys | In wallet manager memory |
| Account state | Not WDK's concern |
| Transaction history | Not WDK's concern |
| External keys | Not supported |

Storage is entirely the responsibility of:
1. The application using WDK (e.g., wrapping with localStorage)
2. Individual wallet implementations

---

## 5. Extension Points

### What you CAN extend

| Extension | How | Example |
|-----------|-----|---------|
| Custom blockchain wallet | `registerWallet(name, WalletClass, config)` | Add ZCash, Cosmos, etc. |
| Custom DeFi protocol | `registerProtocol(blockchain, label, Protocol, config)` | Add Uniswap, 1inch, etc. |
| Account lifecycle hooks | `registerMiddleware(blockchain, fn)` | Logging, analytics, validation |
| Per-account protocols | `account.registerProtocol(label, Protocol, config)` | Account-specific services |

### What you CANNOT extend

| Limitation | Impact |
|-----------|--------|
| 4 fixed protocol types (Swap/Bridge/Lending/Fiat) | Cannot add "Privacy", "Escrow", "ZKProof" protocol type |
| No custom key types at WDK level | Viewing keys, spending keys, ZCash-specific keys — WDK has no concept of these |
| No storage abstraction | Each wallet implementation rolls its own — no shared interface |
| No provider abstraction | No equivalent to Kohaku's `Host.provider` or `Host.network` |
| Seed-derived only | Cannot register accounts from imported keys |

---

## 6. Wallet Ecosystem

| Module | Blockchain | Status |
|--------|-----------|--------|
| `@tetherto/wdk-wallet-evm` | Ethereum, Polygon, Arbitrum | Published |
| `@tetherto/wdk-wallet-evm-erc4337` | EVM (gasless, 4337) | Published |
| `@tetherto/wdk-wallet-ton` | TON | Published |
| `@tetherto/wdk-wallet-ton-gasless` | TON (gasless) | Published |
| `@tetherto/wdk-wallet-btc` | Bitcoin | Published |
| `@tetherto/wdk-wallet-tron` | TRON | Published |
| `@tetherto/wdk-wallet-solana` | Solana | Published |

**No ZCash.** No privacy chains. No shielded transaction support.

---

## 7. Maturity

| Metric | Value |
|--------|-------|
| Version | 1.0.0-beta.5 |
| Commits | 127 |
| Last update | January 2026 |
| Test coverage | 24 test cases (Jest) |
| CI/CD | GitHub Actions (build + publish) |
| Linting | Standard.js |
| Runtime support | Node.js 22+, Bare Node Runtime |
| License | Apache 2.0 |
| Backing | Tether |

---

## 8. OGBank Feasibility Assessment

### Requirements vs. WDK Capabilities

| OGBank Requirement | WDK Can Provide? | Notes |
|-------------------|-------------------|-------|
| Store ZCash viewing key (external, ~128 bytes) | **No** | WDK has no storage layer. Viewing key comes from relayer, not from seed. |
| Derive user_secret from mnemonic | **Partial** | Seed is available, but WDK doesn't expose a `deriveAt(path)` keystore. Would need custom wallet impl. |
| ZCash RPC calls (UTXO scanning) | **No** | No network/fetch abstraction. Each wallet impl handles its own RPC. |
| Avalanche RPC calls (contract interaction) | **Partial** | Could use EVM wallet's provider, but WDK doesn't expose cross-wallet provider sharing. |
| ZK proof generation (Ultrahonk/Noir) | **No** | No proof system support. |
| Escrow account lifecycle | **Partial** | Instance per account works, but "escrow" isn't a wallet concept in WDK. |
| Relayer communication | **No** | No broadcaster pattern. No network abstraction. |
| Cross-chain bridging (ZCash ↔ Avalanche) | **No** | Each blockchain is isolated. No cross-wallet interaction model. |
| Protocol registration for OGBank | **Partial** | Could register as "Lending" protocol, but OGBank's operations (proof generation, escrow, cross-chain) don't fit Swap/Bridge/Lending/Fiat. |

### Fundamental Mismatch

WDK's architecture assumes:
1. **All keys come from the seed.** OGBank's viewing key comes from the relayer.
2. **Wallets are self-contained per blockchain.** OGBank requires ZCash ↔ Avalanche interaction.
3. **Protocols are DeFi services.** OGBank's core operation (ZK proof of cross-chain collateral) isn't a swap, bridge, lending, or fiat operation.
4. **Persistence is someone else's problem.** OGBank's viewing key MUST be persisted securely.

---

## 9. Hypothetical OGBank Adapter

If we forced OGBank into WDK, it would look like:

```javascript
class WalletManagerZcashViewer {
    constructor(seed, config) {
        this._seed = seed;
        this._relayerUrl = config.relayerUrl;
        this._zcashRpcUrl = config.zcashRpcUrl;
        this._viewingKeys = new Map();  // ← must build own storage
    }

    async getAccount(index) {
        // 1. Derive user_secret from seed (custom, not BIP-44)
        // 2. Request escrow from relayer
        // 3. Store viewing key... WHERE?
        //    - WDK provides no storage
        //    - Must implement own persistence (localStorage, IndexedDB, file)
        //    - Must implement own encryption
        // 4. Return account that can:
        //    - Scan escrow balance (via ZCash RPC — must implement own HTTP client)
        //    - Generate ZK proofs (must bring entire Noir/Barretenberg stack)
        //    - Interact with OGBankContract on Avalanche (must get EVM provider from... somewhere)
    }

    dispose() {
        this._viewingKeys.clear();
    }
}

// Registration
wdk.registerWallet('zcash-viewer', WalletManagerZcashViewer, {
    relayerUrl: 'https://relayer.ogbank.io',
    zcashRpcUrl: 'https://zcash.ogbank.io',
});

// Usage
const escrow = await wdk.getAccount('zcash-viewer', 0);
// escrow.getAddress() → escrow ZCash address?
// escrow.sendTransaction() → what transaction? Borrow? Repay? Claim?
```

### Problems with this approach

1. **Storage.** Must build entire persistence + encryption layer from scratch. Kohaku gives `Host.storage` for free.

2. **Cross-chain.** Need Avalanche provider to interact with OGBankContract, but it's registered under a different blockchain ("ethereum" or "avalanche"). WDK isolates blockchains — no way to share providers.

3. **Key import.** `getAccount(index)` implies derivation from seed. But the escrow address and viewing key come from the relayer. The "account" isn't derived — it's imported. The abstraction leaks.

4. **Protocol type.** OGBank isn't a Swap, Bridge, Lending, or Fiat protocol. It's a cross-chain privacy-preserving collateral escrow. None of the 4 types fit. You can shoehorn it into "Lending" but the interface doesn't match (no `borrow()`, `repay()`, `generateProof()` methods).

5. **Proof generation.** Must bring entire Noir/Barretenberg WASM stack inside the wallet implementation. WDK has no circuit or proof abstractions.

6. **Estimated complexity.** ~2000-3000 lines to build what amounts to a standalone OGBank client wrapped in a WDK interface, gaining almost nothing from the WDK layer except the seed phrase.

---

## 10. WDK vs Kohaku — Comparison

| Dimension | Kohaku | WDK |
|-----------|--------|-----|
| **Purpose** | Privacy wallet SDK (plugins for privacy protocols) | Multi-chain wallet manager (orchestrator for DeFi) |
| **Maintainer** | Ethereum Foundation | Tether |
| **Key model** | BIP-32 keystore (`deriveAt(path)`) + external key import via `Host.storage` | BIP-39 seed → BIP-44 derivation. No import. |
| **Can store external keys?** | Yes — `Host.storage.set(key, value)` accepts arbitrary data | No built-in mechanism |
| **Storage abstraction** | `Host.storage` (KV, plaintext) + `Host.keystore` (BIP-32 derived) | None. Delegated to wallet implementations. |
| **Privacy / ZK support** | Core focus: Railgun (Groth16), Privacy Pools, circuit artifacts | None |
| **Cross-chain** | Single-chain (EVM) but `Host.network.fetch()` provides HTTP for any RPC | Multi-chain by design, but each chain is isolated |
| **ZCash support** | No native, but `Host.network` can reach ZCash JSON-RPC | No native, no ZCash wallet implementation exists |
| **Protocol extensibility** | Unlimited — any plugin type via generic `Plugin<>` | 4 fixed types: Swap, Bridge, Lending, Fiat |
| **Provider abstraction** | `Host.provider` (ethers, viem, Colibri, Helios, raw JSON-RPC) | None at WDK level. Each wallet impl brings own. |
| **Network access** | `Host.network.fetch()` — HTTP for circuits, relayer, external APIs | None at WDK level |
| **Broadcaster** | `Broadcaster<TParams, TOperation>` — submit private ops | None |
| **Codebase size** | ~46K lines (Railgun adapter alone) | ~400 lines total |
| **OGBank adapter est.** | ~800 lines (leveraging Host abstractions) | ~2000-3000 lines (must build storage, RPC, crypto from scratch) |
| **Maturity** | Early (WIP stubs, types incomplete, but core interfaces stable) | Beta (v1.0.0-beta.5, clean API, well-tested) |
| **Wallet adoption** | Not yet adopted by major wallets | Used in Tether wallet ecosystem |
| **Account recovery** | `user_secret` from `Host.keystore`, vk re-request from relayer | Seed-only recovery. External keys (vk) cannot be recovered. |

---

## 11. Conclusion

### Kohaku is the right choice for OGBank's viewing key custody.

The comparison isn't close. WDK and Kohaku solve fundamentally different problems.

**WDK** is a wallet orchestrator — it manages accounts derived from a seed phrase across multiple blockchains and attaches DeFi services to them. It's well-designed for its purpose: managing Tether wallets on EVM, TON, Bitcoin, etc. It has no concept of external key storage, privacy primitives, or cross-chain operations that involve proof generation.

**Kohaku** is a privacy wallet SDK — it provides infrastructure for privacy-preserving operations including key storage (`Host.storage`), key derivation (`Host.keystore`), network access (`Host.network`), provider abstraction (`Host.provider`), and a plugin system designed for protocols that generate proofs and handle private state.

OGBank's core requirement is **storing an externally-provided viewing key and using it to generate ZK proofs**. This maps directly to Kohaku's architecture and doesn't map to WDK's at all.

### What WDK could do for OGBank

WDK is not useless to OGBank — it could manage the **Avalanche side** of the user experience. A `@tetherto/wdk-wallet-evm` instance could handle:
- Avalanche wallet connection
- USDC borrow/repay transactions
- Aave V3 interactions (via the existing Aave lending protocol module)
- Fee estimation

But this is the standard EVM wallet part of the stack. It has nothing to do with viewing key custody, which is the critical architectural decision.

### Recommendation

Build the Kohaku adapter as previously recommended in [03_ogbank-kohaku-adapter.md](03_ogbank-kohaku-adapter.md). WDK can optionally be used for the Avalanche wallet management layer, but it does not replace Kohaku's role as the viewing key custodian.

The decision comes down to: who holds the viewing key?

| Option | Who stores vk | Risk |
|--------|-------------|------|
| OGBank servers | OGBank | Single point of total failure (already holds spending key) |
| WDK custom wallet | User's wallet (via custom storage you build yourself) | Must build entire persistence + encryption stack. Gains nothing from WDK. |
| Kohaku plugin | User's wallet (via `Host.storage`) | Storage + keystore + recovery path included. ~800 lines adapter. |
| Raw localStorage | User's browser | No encryption, no backup, no recovery, no ecosystem integration |

Kohaku provides the right abstractions at the right layer. WDK doesn't.

---

## Links

- Kohaku codebase analysis: [01_kohaku-codebase.md](01_kohaku-codebase.md)
- OGBank data requirements: [02_ogbank-data-requirements.md](02_ogbank-data-requirements.md)
- Kohaku adapter feasibility: [03_ogbank-kohaku-adapter.md](03_ogbank-kohaku-adapter.md)
- OGBank protocol spec: [../technical/02_protocol.md](../technical/02_protocol.md)

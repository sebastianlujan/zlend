# AUSD (Agora Dollar) — Analysis for OGBank

> Source: [agora-dollar-evm](https://github.com/amphora-atlas/agora-dollar-evm) codebase + [Agora docs](https://docs.agora.finance/developer).
> Codebase analyzed: February 2026. Local copy at `/agora-dollar-evm`.

**Question:** Can OGBank use AUSD instead of USDC on Avalanche? Does it open a market beyond Morgan without institutional custody dependency?

**Short answer:** AUSD is just as institutional as USDC. VanEck manages the reserves. State Street custodies them. The contract has account freezing. It doesn't solve the custodial stablecoin problem. But it's worth considering as a second borrow asset for ecosystem and market diversification reasons.

---

## 1. What is AUSD

AUSD is a fiat-backed stablecoin issued by **Agora Forge Ltd.** (British Virgin Islands). Minted 1:1 with USD.

| | |
|---|---|
| **Issuer** | Agora Forge Ltd. (BVI) |
| **Asset manager** | VanEck Associates Corporation ($100B AUM) |
| **Custodian** | State Street Bank and Trust Company (G-SIB) |
| **Backing** | Cash, US Treasury bills, overnight reverse repo agreements |
| **Total supply** | ~$134M (Feb 2026) |
| **Chains** | Ethereum, Solana, Avalanche, Polygon, Mantle, Sui, Arbitrum, BNB, Injective, + others (13 networks) |
| **Decimals** | 6 |
| **Inception** | July 7, 2024 |
| **US persons** | Not served. Working on US licensure. |
| **Funding** | $50M Series A led by Paradigm |

Sources: [RWA.xyz](https://app.rwa.xyz/assets/AUSD), [Avalanche blog](https://www.avax.network/about/blog/agora-launches-its-digital-dollar-on-avalanche), [Fortune](https://fortune.com/crypto/2025/07/10/exclusive-agora-stablecoin-series-a-venture-paradigm-crypto-van-eck/)

---

## 2. Codebase Analysis

The `agora-dollar-evm` repo is a Solidity 0.8.28 implementation using Foundry. Well-architected, production-grade.

### Architecture

```
AgoraDollarErc1967Proxy (custom transparent proxy)
    ↓ delegatecall
AgoraDollar (implementation)
    ↓ inherits
AgoraDollarCore
    ↓ inherits
Eip3009 + Erc2612 + Erc20Privileged + AgoraDollarAccessControl
```

### Key contract features

**ERC-20 with extensions:**
- EIP-3009: gas-abstracted transfers (signed authorization)
- EIP-2612: permit (gas-less approvals)
- EIP-712: typed structured data signing
- ERC-7201: namespaced storage (upgrade-safe)

**Proxy optimizations:** Transfer functions (`transfer`, `transferFrom`, `transferWithAuthorization`, `receiveWithAuthorization`) are implemented directly in the proxy for gas savings, with upgrade flags to delegate to the implementation if needed.

### Roles

```solidity
MINTER_ROLE          // Can mint AUSD
BURNER_ROLE          // Can burn AUSD
PAUSER_ROLE          // Can pause transfers, minting, burning, freezing, signatures, bridging
FREEZER_ROLE         // Can freeze/unfreeze individual accounts
BRIDGE_MINTER_ROLE   // Can mint via bridges
BRIDGE_BURNER_ROLE   // Can burn via bridges
ACCESS_CONTROL_MANAGER_ROLE  // Admin — manages all other roles
```

### Account freezing

```solidity
struct Erc20AccountData {
    bool isFrozen;      // Prevents all transfers from/to this account
    uint248 balance;    // Token balance
}
```

- `FREEZER_ROLE` can call `batchFreeze(address[])` / `batchUnfreeze(address[])`
- Frozen accounts cannot transfer OR receive tokens
- Exception: minting TO a frozen account IS allowed (bridge scenarios)
- Global pause via `setIsFreezingPaused()`

### Pause capabilities

The `PAUSER_ROLE` can independently pause:
- All transfers (`isTransferPaused`)
- Minting (`isMintPaused`)
- Burning (`isBurnFromPaused`)
- Account freezing (`isFreezingPaused`)
- Signature verification (`isSignatureVerificationPaused`)
- Bridge operations (`isBridgingPaused`)

All flags are bit-packed into the ERC-1967 implementation storage slot.

---

## 3. AUSD on Avalanche

| Metric | Value |
|--------|-------|
| **Contract** | `0x00000000efe302beaa2b3e6e1b18d08d69a9012a` |
| **Supply on Avalanche** | ~$10M |
| **Holders** | 10,593 |
| **Oracle support** | Chainlink, Pyth, API3, RedStone, Chaos |

### DeFi integrations on Avalanche

| Protocol | Type | Status |
|----------|------|--------|
| **BENQI** | Lending/borrowing | Live — supply + borrow |
| **Trader Joe** | DEX (Liquidity Book) | Live — trading pairs |
| **Pharaoh Exchange** | DEX | Live |
| **Dexalot** | Order book DEX | Live |
| **Wombat Exchange** | Stableswap | Live |
| **Aave V3** | Lending/borrowing | TEMP CHECK passed, ARFC stage (Nov 2024). Status unclear. |

Source: [Avalanche blog](https://www.avax.network/about/blog/agora-launches-its-digital-dollar-on-avalanche), [Aave governance](https://governance.aave.com/t/temp-check-onboard-ausd-to-aave-v3-on-avalanche/19560)

---

## 4. AUSD vs USDC — Honest Comparison

| Dimension | USDC (Circle) | AUSD (Agora) |
|-----------|--------------|--------------|
| **Issuer** | Circle (US regulated) | Agora Forge Ltd. (BVI) |
| **Backing** | Cash + short-term US Treasuries | Cash + US Treasuries + overnight reverse repo |
| **Asset manager** | BlackRock (Circle Reserve Fund) | VanEck |
| **Custodian** | BNY Mellon | State Street |
| **Attestation** | Monthly by Deloitte (Big 4) | Self-reported. Concerns raised in Aave governance about lack of independent verification. |
| **Supply (total)** | ~$45B | ~$134M |
| **Supply (Avalanche)** | Hundreds of millions | ~$10M |
| **Account freezing** | Yes (`blacklist()`) | Yes (`batchFreeze()`) |
| **Transfer pausing** | Yes | Yes (more granular — 6 independent pause flags) |
| **Upgradeability** | Yes (proxy) | Yes (ERC-1967 proxy with selective function upgrades) |
| **US persons** | Served | Not served |
| **DeFi depth (Avalanche)** | Deep — Aave V3, BENQI, Trader Joe, GMX, Platypus, etc. | Early — BENQI, Trader Joe, Pharaoh, Dexalot |
| **Oracle (Avalanche)** | Chainlink (battle-tested) | Chainlink + Pyth + RedStone + API3 (available but less proven for AUSD specifically) |
| **Aave V3 Avalanche** | Live, deep liquidity | Governance proposal in progress |
| **Revenue model** | Circle keeps all yield | Revenue-sharing with distribution partners |

### The "sin instituciones" question

AUSD no resuelve esto. Es tan institucional como USDC:

- **VanEck** ($100B AUM) maneja las reservas
- **State Street** (G-SIB) las custodia
- **Agora Forge Ltd.** controla los roles de minting, burning, freezing
- El contrato tiene `FREEZER_ROLE` que puede congelar cualquier cuenta — igual que el blacklist de USDC
- KYC obligatorio para mint/redeem
- Jurisdicción BVI — menos transparencia regulatoria que Circle en US

Si la preocupación es "no quiero que una institución pueda congelar los fondos de OGBank", AUSD tiene exactamente el mismo riesgo que USDC. `batchFreeze([ogbankContractAddress])` congela todo el protocolo.

---

## 5. OGBank Integration Feasibility

### Opción A: AUSD en Aave V3

El diseño actual de OGBank usa Aave V3: `OGBankContract → approval() → supply() → borrow()`.

- **Estado de AUSD en Aave V3 Avalanche:** TEMP CHECK pasó (823K votos). ARFC en noviembre 2024. Puede que ya esté live o puede que no — requiere verificación.
- **Si está live:** cambiar de USDC a AUSD es trivial — mismo contrato Aave V3, diferente token address.
- **Riesgo:** Liquidez de AUSD en Aave V3 sería baja (supply ~$10M total en Avalanche). Borrows grandes podrían no ser posibles.

### Opción B: AUSD en BENQI

BENQI es el lending protocol nativo de Avalanche. AUSD ya está integrado.

- **Cambio necesario:** Reemplazar la integración Aave V3 por BENQI. Diferentes contratos, diferente interfaz.
- **Ventaja:** Avalanche-native, AUSD ya está live.
- **Desventaja:** Requiere reescribir toda la lógica de lending del OGBankContract. BENQI tiene su propia interfaz (`QiToken`).
- **Implicación:** El diseño actual del protocolo asume Aave V3. Cambiar a BENQI no es trivial.

### Opción C: Soportar ambos (USDC + AUSD)

OGBankContract acepta un parámetro de token al momento del borrow. El usuario elige USDC o AUSD.

- **Complejidad:** Doble integración, doble oracle, doble liquidez assessment.
- **Ventaja:** Diversificación de riesgo de stablecoin. Si Circle congela a OGBank, AUSD sigue funcionando (y viceversa).
- **Para MVP:** Probablemente excesivo. Mejor empezar con uno y agregar el otro después.

---

## 6. What AUSD Actually Offers OGBank

Aunque AUSD no resuelve el problema institucional, sí ofrece algunas ventajas estratégicas:

### Revenue sharing
Agora comparte revenue con distribution partners. Si OGBank genera volumen de AUSD, Agora podría ser un partner estratégico que ayuda con:
- Grants o incentivos
- Liquidity provisioning
- Co-marketing en el ecosistema Avalanche

### Avalanche ecosystem alignment
AUSD fue lanzado con soporte directo de Avalanche Foundation. Integrar AUSD señala alineamiento con el ecosistema y podría facilitar:
- Avalanche Foundation grants
- Integración con otros protocols del ecosistema
- Visibilidad en el Avalanche Builder Hub

### Non-US market
AUSD no sirve a US persons. Esto podría ser una ventaja para OGBank:
- ZCash holders fuera de US (mercado más grande que solo US)
- Menos fricción regulatoria con una stablecoin que no tiene US compliance overhead
- Complementario si OGBank decide no servir US market inicialmente

### Stablecoin diversification
Si OGBank soporta AUSD + USDC:
- Reduce riesgo de single-stablecoin dependency
- Si Circle blacklists la address de OGBank, AUSD sigue funcionando
- Pero ambos tienen freezing — si la presión regulatoria viene por el lado privacy/ZCash, ambos podrían freezar

---

## 7. Risks Specific to AUSD

| Risk | Severity | Notes |
|------|----------|-------|
| **Low liquidity** | High | $10M en Avalanche. Un borrow de $500K ya impacta la liquidez disponible. |
| **Attestation transparency** | Medium | Aave governance raised concerns: no independent auditor, self-reported reserves. |
| **BVI jurisdiction** | Medium | Menos protección legal que Circle (US regulated). Bankruptcy remoteness via Delaware Statutory Trust, pero no probado. |
| **Not on Aave V3 (possibly)** | High | Si AUSD no está live en Aave V3, la integración actual de OGBank no funciona. Requiere BENQI u otro lending protocol. |
| **Oracle immaturity** | Medium | Chainlink soporta AUSD, pero con menos historial que USDC. Flash crash + thin oracle = liquidation risk. |
| **Depeg risk** | Low-Medium | $134M supply es chico. Un redeem run podría despeggear. USDC ($45B) tiene mucho más buffer. |

---

## 8. Recommendation

### Para MVP (Phase 1): USDC en Aave V3

- Liquidez profunda, oracle probado, Aave V3 battle-tested, Morgan entiende USDC.
- No agregar complejidad innecesaria al MVP.

### Para Phase 2: Evaluar AUSD como segundo borrow asset

- Si AUSD está live en Aave V3 para ese momento → trivial agregar.
- Si no → evaluar BENQI como alternativa (requiere más trabajo).
- Revenue-sharing con Agora podría ser estratégicamente valioso.
- Diversificación de stablecoin reduce single-point-of-failure risk.

### Lo que AUSD NO resuelve

AUSD no elimina el riesgo institucional. Si el objetivo es "un stablecoin que nadie puede congelar", la respuesta no es AUSD — es un stablecoin descentralizado (DAI, LUSD, RAI). Pero esos tienen sus propios problemas (volatilidad del colateral, menor liquidez, sin Avalanche deployment maduro).

El problema de account freezing es existencial para OGBank con CUALQUIER stablecoin centralizado. Si Circle o Agora congelan la address del OGBankContract, el protocolo se detiene. Esto debería documentarse como riesgo en la arquitectura independientemente de qué stablecoin se elija.

---

## Links

- Agora landing: [agora.finance](https://www.agora.finance/)
- Agora docs: [docs.agora.finance](https://docs.agora.finance/developer)
- AUSD metrics: [RWA.xyz](https://app.rwa.xyz/assets/AUSD)
- AUSD on Avalanche: [Avalanche blog](https://www.avax.network/about/blog/agora-launches-its-digital-dollar-on-avalanche)
- Aave governance proposal: [TEMP CHECK](https://governance.aave.com/t/temp-check-onboard-ausd-to-aave-v3-on-avalanche/19560)
- BENQI: [benqi.fi](https://benqi.fi/)
- OGBank architecture: [04_architecture.md](../product/04_architecture.md)
- OGBank protocol spec: [../technical/02_protocol.md](../technical/02_protocol.md)

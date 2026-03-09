# ZLend (OGBank) — UX Design Document

## 1. Overview

ZLend es un protocolo de privacidad que conecta Zcash con Avalanche. Los usuarios depositan ZEC en Zcash, generan pruebas ZK, y obtienen prestamos USDT en Avalanche via Aave V3 — sin revelar su identidad.

### Modelo mental del usuario

El usuario piensa en terminos de un ciclo simple:

```
"Pongo ZEC" → "Saco USDT" → "Devuelvo USDT" → "Recupero ZEC"
```

La UI debe mapear exactamente a este modelo mental, sin exponer complejidad tecnica innecesaria (nullifiers, Aave pools, ZK circuits).

---

## 2. Analisis Don Norman — Los 6 principios aplicados

### 2.1 Visibilidad (Visibility of System Status)

**Problema**: El protocolo tiene un estado complejo (nullifier chain de 3 niveles). El usuario necesita saber en que paso esta sin entender la mecanica interna.

**Solucion**: Un **LifecycleTracker** (stepper horizontal) siempre visible que muestra:

```
  [1 Deposit] ——> [2 Generate Proof] ——> [3 Borrow] ——> [4 Repay] ——> [5 Withdraw]
      ●                  ○                    ○              ○              ○
   completed           current              locked         locked         locked
```

- Steps completados: circulo lleno + color primary
- Step actual: circulo con borde animado (pulse)
- Steps bloqueados: circulo gris + linea punteada

Ademas un **SummaryStrip** permanente muestra numeros clave:
```
  ZEC Deposited: 2.5    |    USDT Borrowed: 1,200    |    Active Loans: 1    |    Network: Anvil Local
```

### 2.2 Feedback

**Problema**: Las transacciones on-chain tardan segundos. El usuario necesita saber que paso en cada momento.

**Solucion**: Ciclo de feedback de 4 estados para toda accion:

```
  [Idle] → click → [Submitting...] → tx hash → [Confirming...] → receipt → [Success ✓]
                                                                          → [Failed ✗]
```

Implementado como:
- **Boton**: cambia texto + muestra spinner inline
- **Toast**: aparece con tx hash linkeable a explorer
- **Card**: actualiza estado visual inmediatamente tras confirmacion

Para operaciones mocked (deposit ZEC, generate proof): delay artificial de 1.5s con spinner para que se sienta real, seguido de confirmacion inmediata.

### 2.3 Affordance

**Problema**: El usuario debe entender que puede hacer en cada momento sin leer instrucciones.

**Solucion**:

| Elemento | Affordance visual |
|----------|------------------|
| Boton Primary (rojo) | Accion principal, avanza el ciclo |
| Boton Secondary (borde) | Acciones secundarias (copy, refresh) |
| Boton Ghost (transparente) | Navegacion, info |
| Boton Disabled (gris) | No disponible — tooltip explica por que |
| Card con border-dashed | Operacion simulada/mocked |
| Badge "Simulated" | Esta operacion no es real aun |
| Input con MAX button | Puedo usar todo mi saldo |
| Monto read-only (sin input) | No puedo modificar este valor |

### 2.4 Constraints (Restricciones)

**Problema critico**: `repay()` consume el nullifier en UNA sola llamada. Si el usuario paga parcial, pierde la vault para siempre.

**Solucion UX**:
1. **NO hay input de monto** en el repay — el sistema calcula el total exacto (principal + interes Aave) y lo muestra como valor fijo
2. **Alert rojo** permanente: "El repago debe hacerse completo en una sola transaccion"
3. **Modal de confirmacion** obligatorio antes de ejecutar
4. **Balance check**: si el usuario no tiene suficiente USDT, el boton esta disabled con mensaje claro

Otras constraints:
- Borrow disabled hasta que proof este generada
- Withdraw disabled hasta que repay este confirmado
- Tabs muestran estado vacio con CTA si no hay vaults/loans

### 2.5 Mapping (Correspondencia natural)

**Solucion**: Los tabs siguen el orden cronologico del ciclo de vida:

```
  [Deposit]  →  [Borrow]  →  [Repay & Withdraw]
   izquierda     centro         derecha
   primero      segundo          ultimo
```

Dentro de cada tab, los componentes estan ordenados de arriba a abajo en el orden en que se ejecutan:

```
BorrowView:
  1. Vault Selector    (primero elijo vault)
  2. Generate Proof    (despues genero proof)
  3. Borrow Form       (despues pido prestamo)
  4. Active Loans      (veo resultado abajo)
```

### 2.6 Consistency

**Solucion**: Patrones visuales uniformes:

| Patron | Donde se usa |
|--------|-------------|
| Card con bg-surface-900/50 | Todo contenedor de contenido |
| Badge pill gris | Estado, mock indicator, network |
| Texto surface-400 uppercase tracking-wider | Labels |
| Font mono text-2xl bold white | Valores numericos grandes |
| Font mono text-sm surface-400 | Simbolos de token |
| border-dashed border-surface-600 | Operaciones mocked |
| Primary button rojo | Accion forward en cada paso |

---

## 3. Flujos de usuario

### 3.1 Flujo completo (happy path)

```
USUARIO NUEVO (wallet conectada, sin vaults)
│
├─ Tab "Deposit" (default)
│  ├─ Ve mensaje: "Start by depositing ZEC to create your first vault"
│  ├─ Click "Get Deposit Address" → genera direccion shielded (mock)
│  ├─ Copia direccion
│  ├─ Ingresa monto ZEC → Click "Deposit ZEC" → spinner 1.5s → vault creada
│  └─ Vault aparece en VaultList con status "Deposited"
│
├─ Tab "Borrow"
│  ├─ Selecciona vault del dropdown
│  ├─ Click "Generate ZK Proof" → spinner 2s → proof ready (mock)
│  ├─ Ingresa monto USDT a pedir
│  ├─ Click "Borrow USDT" → tx submitted → confirming → USDT en wallet
│  └─ Loan aparece en ActiveLoansTable
│
└─ Tab "Repay & Withdraw"
   ├─ Selecciona loan del dropdown
   ├─ Ve monto total (auto-calculado): "1,200.45 USDT"
   ├─ Ve Alert rojo: "Must repay full amount in one transaction"
   ├─ Click "Approve USDT" (si necesario) → tx
   ├─ Click "Repay Full Amount" → Modal confirmacion → tx → loan repaid
   ├─ WithdrawCard se habilita
   ├─ Click "Withdraw ZEC" → tx → FinishPayment emitido
   └─ Ve status "Complete — relayer will return 2.5 ZEC to your shielded address"
```

### 3.2 Flujo de error: balance insuficiente para repay

```
Tab "Repay & Withdraw"
├─ Total due: 1,200.45 USDT
├─ User balance: 800.00 USDT
├─ Boton "Repay" → DISABLED
├─ Tooltip: "Insufficient balance. You need 1,200.45 USDT (you have 800.00)"
└─ Accion sugerida: "Get more USDT to repay your loan"
```

### 3.3 Flujo: multiples vaults

```
VaultList muestra todas las vaults:
┌─────────────────────────────────────────────────┐
│ Vault #1    2.5 ZEC    ● Borrowed    3 days ago │
│ Vault #2    1.0 ZEC    ● Deposited   1 hour ago │
│ Vault #3    5.0 ZEC    ✓ Withdrawn   1 week ago │
└─────────────────────────────────────────────────┘

Cada vault es independiente con su propio nullifier chain.
El dropdown en cada tab filtra por status relevante:
- Borrow tab: solo vaults DEPOSITED o PROOF_READY
- Repay tab: solo vaults BORROWED
- Withdraw section: solo vaults REPAID
```

---

## 4. Wireframes ASCII

### 4.1 Layout general

```
┌──────────────────────────────────────────────────────────────────────┐
│  OGBank                    [Deposit] [Borrow] [Repay]    [0x1a..3f] │
├──────────────────────────────────────────────────────────────────────┤
│  ZEC: 3.5  │  USDT Borrowed: 1,200  │  Loans: 1  │  Anvil Local    │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│                       [ Tab Content Area ]                           │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 4.2 Deposit Tab

```
┌──────────────────────────────────────────────────────────────────────┐
│                                                                      │
│  [● Deposit] ─── [○ Proof] ─── [○ Borrow] ─── [○ Repay] ─── [○ Withdraw]
│                                                                      │
│  ┌─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┐  ┌─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┐ │
│  ╎  DEPOSIT ADDRESS   [Simulated]   ╎  ╎  DEPOSIT ZEC           ╎ │
│  ╎                                   ╎  ╎  [Simulated]           ╎ │
│  ╎  zs1qr4...7xk2                   ╎  ╎                        ╎ │
│  ╎                          [Copy]   ╎  ╎  Amount  [____] ZEC    ╎ │
│  ╎                                   ╎  ╎                        ╎ │
│  ╎  [Get New Address]                ╎  ╎  [Deposit ZEC]         ╎ │
│  └─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┘  └─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┘ │
│                                                                      │
│  YOUR VAULTS                                                         │
│  ┌──────────────────────────────────────────────────────────────────┐│
│  │  #1   2.5 ZEC   ● Deposited   2 hours ago                      ││
│  │  #2   1.0 ZEC   ✓ Withdrawn   3 days ago                       ││
│  └──────────────────────────────────────────────────────────────────┘│
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

Notas:
- Border punteado (`─ ─ ─`) indica cards con operaciones mocked/simulated
- Badge "Simulated" en cada card mocked
- Dos cards lado a lado: Address (izq) + Deposit Form (der)
- VaultList abajo con todas las vaults y su status

### 4.3 Borrow Tab

```
┌──────────────────────────────────────────────────────────────────────┐
│                                                                      │
│  [✓ Deposit] ─── [● Proof] ─── [○ Borrow] ─── [○ Repay] ─── [○ Withdraw]
│                                                                      │
│  Select Vault  [▼ Vault #1 — 2.5 ZEC                            ]   │
│                                                                      │
│  ┌─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┐ │
│  ╎  GENERATE ZK PROOF                            [Simulated]     ╎ │
│  ╎                                                                ╎ │
│  ╎  Status: Ready ✓                                               ╎ │
│  ╎                                                                ╎ │
│  ╎  [Generate Proof]                                              ╎ │
│  └─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┘ │
│                                                                      │
│  ┌───────────────────────────────────────────────────────────────────┐│
│  │  BORROW USDT                                                     ││
│  │                                                                   ││
│  │  Amount     [________] USDT                                       ││
│  │  Rate       Aave Variable ~3.2%                                   ││
│  │  Collateral 2.5 ZEC backing this loan                             ││
│  │                                                                   ││
│  │  [Borrow USDT]                                                    ││
│  └───────────────────────────────────────────────────────────────────┘│
│                                                                      │
│  ACTIVE LOANS                                                        │
│  ┌───────────────────────────────────────────────────────────────────┐│
│  │  Vault   Principal    Interest    Total Due    Status             ││
│  │  #1      1,200 USDT   0.45 USDT   1,200.45    ● Active          ││
│  └───────────────────────────────────────────────────────────────────┘│
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

Notas:
- VaultSelector como dropdown arriba de todo
- ProofGenerationCard con border punteado (mocked)
- BorrowForm con border solido (real tx)
- ActiveLoansTable abajo muestra loans activos

### 4.4 Repay & Withdraw Tab

```
┌──────────────────────────────────────────────────────────────────────┐
│                                                                      │
│  [✓ Deposit] ─── [✓ Proof] ─── [✓ Borrow] ─── [● Repay] ─── [○ Withdraw]
│                                                                      │
│  Select Loan  [▼ Vault #1 — 1,200.45 USDT due                   ]   │
│                                                                      │
│  ┌───────────────────────────────────────────────────────────────────┐│
│  │  REPAY LOAN                                                       ││
│  │                                                                   ││
│  │  ┌─────────────────────────────────────────────────────────────┐  ││
│  │  │  ⚠ IMPORTANT: Repayment must be made in FULL in a single   │  ││
│  │  │  transaction. Partial repayment will permanently lock your  │  ││
│  │  │  vault. This action cannot be undone.                       │  ││
│  │  └─────────────────────────────────────────────────────────────┘  ││
│  │                                                                   ││
│  │  Principal         1,200.00 USDT                                  ││
│  │  Accrued Interest      0.45 USDT                                  ││
│  │  ─────────────────────────────                                    ││
│  │  Total Due         1,200.45 USDT                                  ││
│  │                                                                   ││
│  │  Your Balance      2,500.00 USDT  ✓ Sufficient                    ││
│  │                                                                   ││
│  │  [Approve USDT]   [Repay Full Amount (1,200.45 USDT)]            ││
│  └───────────────────────────────────────────────────────────────────┘│
│                                                                      │
│  ┌─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┐ │
│  ╎  WITHDRAW ZEC                                  [Simulated]     ╎ │
│  ╎                                                                ╎ │
│  ╎  Amount     [________] ZEC                                     ╎ │
│  ╎                                                                ╎ │
│  ╎  The relayer will return ZEC to your shielded address          ╎ │
│  ╎  zs1qr4...7xk2                                                ╎ │
│  ╎                                                                ╎ │
│  ╎  [Withdraw ZEC]          (disabled until repaid)               ╎ │
│  └─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┘ │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

Notas:
- Alert rojo prominente sobre one-shot repay
- Monto total como read-only (NO input)
- Balance check visible con indicador verde/rojo
- Approve + Repay como dos botones separados
- WithdrawCard con border punteado (proof generation mocked)
- WithdrawCard disabled hasta que repay confirme

### 4.5 Modal de confirmacion (Repay)

```
┌───────────────────────────────────────────┐
│                                           │
│  Confirm Full Repayment                   │
│                                           │
│  You are about to repay:                  │
│                                           │
│      1,200.45 USDT                        │
│                                           │
│  This is the FULL amount including        │
│  accrued interest. After repayment,       │
│  you can withdraw your ZEC.               │
│                                           │
│  ⚠ This action cannot be undone.          │
│                                           │
│  [Cancel]         [Confirm Repayment]     │
│                                           │
└───────────────────────────────────────────┘
```

### 4.6 Toast de transaccion

```
┌───────────────────────────────────────┐
│  ● Transaction Submitted              │
│  Repaying 1,200.45 USDT...           │
│  View on Explorer ↗                   │
└───────────────────────────────────────┘

     ↓ (tras confirmacion)

┌───────────────────────────────────────┐
│  ✓ Repayment Confirmed               │
│  1,200.45 USDT repaid successfully   │
│  View on Explorer ↗                   │
└───────────────────────────────────────┘
```

### 4.7 Estado vacio (sin wallet)

```
┌──────────────────────────────────────────────────────────────────────┐
│                                                                      │
│                                                                      │
│                    Connect your wallet to get started                 │
│                                                                      │
│                    ZLend lets you borrow USDT against                 │
│                    your ZEC holdings — privately.                     │
│                                                                      │
│                    [Connect Wallet]                                   │
│                                                                      │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 4.8 Estado vacio (wallet conectada, sin vaults)

```
Tab "Deposit":
┌──────────────────────────────────────────────────────────────────────┐
│                                                                      │
│  Start by depositing ZEC to create your first vault.                 │
│                                                                      │
│  ┌─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┐ │
│  ╎  GET DEPOSIT ADDRESS               [Simulated]                ╎ │
│  ╎  ...                                                           ╎ │

Tab "Borrow" (sin vaults):
┌──────────────────────────────────────────────────────────────────────┐
│                                                                      │
│  No vaults available. Deposit ZEC first to start borrowing.          │
│                                                                      │
│  [Go to Deposit →]                                                   │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 5. Componentes: jerarquia completa

```
App
├── WagmiProvider + QueryClientProvider
├── Header
│   ├── Logo "OGBank"
│   ├── TabNav [Deposit | Borrow | Repay & Withdraw]
│   └── ConnectButton
├── SummaryStrip
│   └── StatItem[] (ZEC deposited, USDT borrowed, Active loans, Network)
│
├── DepositView
│   ├── LifecycleTracker (step=1)
│   ├── Grid 2-col:
│   │   ├── DepositAddressCard [mock, dashed border]
│   │   │   ├── Badge "Simulated"
│   │   │   ├── Address display + Copy button
│   │   │   └── Button "Get New Address" (secondary)
│   │   └── DepositForm [mock, dashed border]
│   │       ├── Badge "Simulated"
│   │       ├── AmountInput (ZEC)
│   │       └── Button "Deposit ZEC" (primary)
│   └── VaultList
│       └── VaultCard[]
│           ├── Vault ID + ZEC amount
│           ├── Badge (status)
│           └── Timestamp
│
├── BorrowView
│   ├── LifecycleTracker (step=3)
│   ├── VaultSelector (dropdown)
│   ├── ProofGenerationCard [mock, dashed border]
│   │   ├── Badge "Simulated"
│   │   ├── Status indicator
│   │   └── Button "Generate ZK Proof" (primary)
│   ├── BorrowForm [solid border — real tx]
│   │   ├── AmountInput (USDT)
│   │   ├── InfoRow "Aave Variable Rate"
│   │   ├── InfoRow "Vault Collateral"
│   │   └── Button "Borrow USDT" (primary, disabled until proof)
│   └── ActiveLoansTable
│       └── LoanRow[]
│           ├── Vault ref + Principal + Interest + Total
│           └── Badge (Active/Repaid/Withdrawn)
│
├── RepayWithdrawView
│   ├── LifecycleTracker (step=4 or 5)
│   ├── LoanSelector (dropdown)
│   ├── RepayCard [solid border — real tx]
│   │   ├── Alert (danger — one-shot warning)
│   │   ├── InfoRow "Principal"
│   │   ├── InfoRow "Accrued Interest"
│   │   ├── InfoRow "Total Due" (bold)
│   │   ├── InfoRow "Your Balance" + sufficient/insufficient indicator
│   │   ├── Button "Approve USDT" (secondary, if needed)
│   │   └── Button "Repay Full Amount (X.XX USDT)" (primary)
│   ├── WithdrawCard [mock proof, dashed border]
│   │   ├── Badge "Simulated"
│   │   ├── AmountInput (ZEC)
│   │   ├── InfoRow "Relayer return address"
│   │   └── Button "Withdraw ZEC" (primary, disabled until repaid)
│   └── Modal (repay confirmation)
│
└── TransactionToast (global, portal)
    ├── Status icon (spinner/check/x)
    ├── Message
    └── Explorer link
```

---

## 6. State machine por Vault

```
                    ┌──────────┐
                    │ DEPOSITED │
                    └────┬─────┘
                         │ generate proof (mock)
                         ▼
                  ┌─────────────┐
                  │ PROOF_READY │
                  └──────┬──────┘
                         │ borrow() tx confirmed
                         ▼
                    ┌──────────┐
                    │ BORROWED │
                    └────┬─────┘
                         │ repay() tx confirmed
                         ▼
                    ┌─────────┐
                    │ REPAID  │
                    └────┬────┘
                         │ withdrawProof() tx confirmed
                         ▼
                   ┌───────────┐
                   │ WITHDRAWN │
                   └───────────┘
```

Cada transicion requiere la anterior. No se puede saltar pasos.
El UI enforcea esto deshabilitando acciones fuera de orden.

---

## 7. Tratamiento visual: Mock vs Real

| Aspecto | Operacion Mock | Operacion Real |
|---------|---------------|----------------|
| Border | `border-dashed border-surface-600` | `border border-surface-700/50` (default Card) |
| Badge | Badge "Simulated" (bg-surface-700 text-surface-400) | Sin badge extra |
| Feedback | Delay artificial 1.5-2s + spinner local | Tx lifecycle real (submit → confirm) |
| Ejemplo | Deposit ZEC, Generate Proof | Borrow USDT, Repay, Withdraw |

---

## 8. Paleta de colores (existente, reutilizar)

| Uso | Color | Tailwind class |
|-----|-------|---------------|
| Accion principal | #FF394A | `bg-primary-500` |
| Accion principal hover | #E84142 | `bg-primary-600` |
| Acento/info | #058AFF | `text-accent-500` |
| Texto principal | white | `text-white` |
| Texto secundario | #9c9ca6 | `text-surface-400` |
| Texto terciario | #6e6e7a | `text-surface-500` |
| Labels | #9c9ca6 | `text-surface-400 uppercase tracking-wider` |
| Valores numericos | white, mono | `font-mono text-2xl font-bold text-white` |
| Border cards | #333340/50% | `border-surface-700/50` |
| Background cards | #161617/50% | `bg-surface-900/50` |
| Background page | #0a0a0c | `bg-surface-950` |
| Warning/danger | #FF394A | `border-primary-500 bg-primary-500/10` |
| Success | green | `text-green-400` |

---

## 9. Responsive breakpoints

| Breakpoint | Layout |
|------------|--------|
| Mobile (<640px) | Stack todo vertical, tabs como iconos, 1 columna |
| Tablet (640-1024px) | 2 columnas donde aplique, tabs con texto |
| Desktop (>1024px) | Layout completo, max-w-5xl centrado |

---

## 10. Resumen de decisiones clave

1. **Tabs, no pages** — Single-page con 3 tabs que mapean al ciclo de vida
2. **Repay auto-calculado** — Sin input manual, sistema calcula total exacto
3. **Mock = dashed border + badge** — Consistente en toda la app
4. **LifecycleTracker en cada tab** — Siempre visible el progreso
5. **Vaults en localStorage** — Estado local por wallet address (las ops ZEC son mock)
6. **Loans derivados on-chain** — useReadContracts para verificar estado de nullifiers
7. **No router** — useState simple para tabs
8. **No state library** — React state + wagmi hooks + React Query suficiente

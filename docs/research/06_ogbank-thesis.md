# OGBank: Delegated Spending Vaults for Shielded Cryptocurrency

**Abstract.** Privacy-preserving cryptocurrencies like Zcash provide strong confidentiality guarantees through zero-knowledge proofs, but their spend authorization model is binary — possession of a secret key grants unbounded spending power. We present OGBank, a protocol for constructing delegated spending vaults within Zcash's Orchard shielded pool that requires no consensus-level changes. OGBank combines three primitives: (1) threshold signature key splitting via Re-Randomized FROST over a hierarchically derived vault address, (2) a Tornado Cash-inspired authorization commitment scheme with single-use nullifiers that cryptographically bounds delegated spending parameters, and (3) an oblivious chain synchronization model that prevents the delegated party from learning the vault's transaction history. The resulting system enables an owner to pre-authorize a relayer to execute bounded spends from a shielded vault while remaining offline, with on-chain transactions indistinguishable from standard single-party shielded transfers.

---

## 1. Introduction

Shielded transaction protocols [1, 2] achieve ledger indistinguishability: transactions appear as opaque ciphertext paired with a zero-knowledge proof of validity. This property provides confidentiality far stronger than decoy-based approaches [3], but it comes at the cost of programmability. In transparent blockchain systems such as Ethereum, the ERC-20 `approve/transferFrom` pattern enables one address to authorize another to spend tokens on its behalf, subject to an on-chain allowance. This primitive underlies decentralized finance: lending protocols require collateral lockup managed by smart contracts, and treasury management demands role-based access with bounded spending authority.

No equivalent primitive exists for shielded transactions. In Zcash's Orchard protocol [2], spend authorization is determined solely by knowledge of a spend authorizing key `ask`. There is no mechanism to express "address A authorizes address B to spend at most V coins to destination D before block height H" — the conditions that would constitute an allowance. The Orchard Action circuit enforces balance integrity and spend authorization, but cannot express conditional spending logic without circuit modifications requiring a network upgrade.

We observe that the allowance problem decomposes into two sub-problems with different solutions. The *authorization splitting* problem — enabling multiple parties to jointly control spending — is solved by threshold signatures. The *authorization bounding* problem — constraining what a delegate can do — is an application-layer concern that can be addressed through pre-committed spending tickets with cryptographic replay prevention.

OGBank combines these solutions into a complete delegated vault protocol. Our contributions are:

1. A construction for Orchard shielded vaults using ZIP-32 hierarchical derivation with FROST threshold key splitting, where the spend authorizing key never exists as a single value.

2. An authorization commitment scheme — directly inspired by Tornado Cash's deposit-nullifier pattern [4] — that pre-commits specific spending parameters into a Merkle tree. Each authorization is single-use, enforced by nullifiers, and cryptographically bounds the delegate's spending power.

3. An oblivious synchronization architecture, drawing on techniques proposed by Bowe [5], that enables chain state tracking without exposing the vault's viewing key to the delegate.

4. A proof-carrying balance attestation that enables cross-chain collateral verification without trusted intermediaries.

The protocol requires no changes to Zcash consensus rules. On-chain, all vault operations produce standard Orchard transactions indistinguishable from single-party shielded transfers.

## 2. Preliminaries

### 2.1 Orchard Shielded Protocol

We work within Zcash's Orchard shielded pool [2], which operates over the Pallas/Vesta curve cycle. An Orchard note is a tuple $(d, pk_d, v, \rho, \psi, rcm)$ representing a payment of value $v$ to the diversified payment address $(d, pk_d)$.

**Key structure.** From a 32-byte spending key $sk$, Orchard derives:
- $ask = \text{ToScalar}(\text{PRF}^{\text{expand}}(sk, [0\text{x}06]))$ — the spend authorizing key
- $nk = \text{ToBase}(\text{PRF}^{\text{expand}}(sk, [0\text{x}07]))$ — the nullifier deriving key
- $rivk = \text{ToScalar}(\text{PRF}^{\text{expand}}(sk, [0\text{x}08]))$ — commitment randomness key
- $ak = [ask] \cdot \mathcal{G}^{\text{Orchard}}$ — the spend validating key (public)
- $ivk = \text{Commit}^{ivk}_{rivk}(ak, nk)$ — the incoming viewing key

The full viewing key $fvk = (ak, nk, rivk)$ enables detection of incoming/outgoing transactions and computation of nullifiers, but cannot authorize spending.

**Spend authorization.** To spend a note, a transaction reveals a nullifier $nf$ (derived deterministically from the note and $nk$), and provides: (a) a Halo 2 zero-knowledge proof [6] over the Vesta curve demonstrating note existence in the commitment tree, nullifier integrity, value commitment integrity, and key consistency; and (b) a RedPallas [7] signature $\sigma$ under a re-randomized key $rk = [ask + \alpha] \cdot \mathcal{G}$ for a fresh randomizer $\alpha$.

The critical observation is that proof generation and spend authorization are independent. The proof requires $(fvk, \text{note}, \text{merkle\_path})$; the signature requires only $ask$ and $\alpha$. This separation enables threshold signing without circuit modification.

### 2.2 Hierarchical Deterministic Derivation

ZIP-32 [8] defines hierarchical deterministic key derivation for Orchard using hardened-only child key generation. From a master seed $S$, the derivation path $m_{\text{Orchard}} / 32' / 133' / \text{account}'$ produces independent spending keys at each account index. Each child derivation applies $\text{PRF}^{\text{expand}}(c_{\text{par}}, [0\text{x}81] \| sk_{\text{par}} \| i)$ to produce a new $(sk, c)$ pair, where $c$ is a 32-byte chain code.

### 2.3 Threshold Signatures (FROST)

FROST [9] is a threshold Schnorr signature scheme that enables $t$-of-$n$ signing. Re-Randomized FROST [10], specified for Zcash in ZIP-312 [11], extends FROST to produce signatures under re-randomizable keys, as required by Orchard's spend authorization.

In a $t$-of-$n$ FROST configuration, the secret key $ask$ is $(t,n)$-Shamir-shared among $n$ participants. Any $t$ participants can jointly produce a valid signature, but $t-1$ participants cannot. The resulting signature is indistinguishable from a single-party Schnorr signature.

Distributed Key Generation (DKG) [12] allows participants to jointly generate shares of $ask$ without any party ever holding the full key. Each participant $P_i$ generates a random polynomial $f_i(x)$ of degree $t-1$, commits to its coefficients, and distributes evaluations to other participants. The group public key $ak = \sum_i [a_{i,0}] \cdot \mathcal{G}$ is computed from the constant-term commitments.

### 2.4 Commitment-Nullifier Schemes

Tornado Cash [4] introduced a practical commitment-nullifier scheme for privacy-preserving withdrawals. A depositor commits $C = H(k \| r)$ for random nullifier $k$ and secret $r$, inserting $C$ into an on-chain Merkle tree. To withdraw, the depositor reveals $h = H(k)$ (the nullifier hash) and provides a zero-knowledge proof that they know $(k, r)$ such that $H(k \| r)$ is in the tree and $H(k) = h$. The contract verifies the proof, checks $h$ has not been seen before, and transfers funds. The withdrawal includes the recipient address as a public circuit input, preventing front-running.

The key property we borrow is that authorization parameters are cryptographically bound at commitment time and cannot be modified at redemption time.

## 3. Protocol Construction

### 3.1 Vault Derivation

A vault is an Orchard sub-account at a deterministic ZIP-32 derivation path. Given owner seed $S$ and vault nonce $\eta \in [0, 2^{32})$:

$$\text{vault\_index} = H_{\text{idx}}(\text{SeedFP}(S) \| \eta) \mod 2^{31}$$

where $H_{\text{idx}}$ is BLAKE2b-256 with personalization `"OGBank__VaultIdx"` an

<!-- TRUNCATED: Content beyond this point was cut off during submission. Sections 3.2+ pending. -->

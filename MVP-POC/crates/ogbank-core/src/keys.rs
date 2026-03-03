// Phase 1: ZIP-32 Key Derivation
//
// Derives the full OGBank key hierarchy from a BIP-39 mnemonic:
//   seed → sk → {ask, nk, rivk} → ak → fvk → {ivk, ovk, dk} → address(d)
//
// Reference: docs/technical/05_zcash-integration.md, docs/technical/08_mvp.md

use orchard::keys::{
    FullViewingKey, IncomingViewingKey, OutgoingViewingKey, Scope, SpendingKey,
};
use orchard::Address;
use zip32::{AccountId, DiversifierIndex};

/// Zcash mainnet coin type (BIP-44: 133).
const ZCASH_COIN_TYPE: u32 = 133;

/// OGBank account index. ASCII "OG" = 0x4F47 = 20295.
/// Used as the account index in the ZIP-32 derivation path: m/32'/133'/20295'
const OGBANK_ACCOUNT_INDEX: u32 = 0x4F47;

#[derive(Debug, thiserror::Error)]
pub enum KeyError {
    #[error("invalid mnemonic: {0}")]
    InvalidMnemonic(String),

    #[error("key derivation failed: {0}")]
    DerivationFailed(String),

    #[error("invalid seed length: expected 64 bytes, got {0}")]
    InvalidSeedLength(usize),
}

/// Complete OGBank key set derived from a BIP-39 mnemonic via ZIP-32.
///
/// Contains all keys needed for the OGBank protocol lifecycle:
/// - `sk`: spending key (CRITICAL — can derive all other keys, signs transactions)
/// - `fvk`: full viewing key (shared with relayer for balance verification)
/// - `ivk`: incoming viewing key (for trial decryption of received notes)
/// - `ovk`: outgoing viewing key (for viewing sent transactions)
/// - `address`: the Orchard shielded address that receives collateral deposits
pub struct OGBankKeys {
    pub mnemonic: String,
    pub sk: SpendingKey,
    pub fvk: FullViewingKey,
    pub ivk: IncomingViewingKey,
    pub ovk: OutgoingViewingKey,
    pub address: Address,
}

impl std::fmt::Debug for OGBankKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OGBankKeys")
            .field("mnemonic", &"[REDACTED]")
            .field("sk", &"[REDACTED]")
            .field("address", &hex::encode(self.address.to_raw_address_bytes()))
            .finish()
    }
}

impl OGBankKeys {
    /// Generate a new random OGBank identity.
    ///
    /// Creates a fresh 24-word BIP-39 mnemonic and derives all keys.
    pub fn generate() -> Result<Self, KeyError> {
        let mnemonic = bip39::Mnemonic::generate(24)
            .map_err(|e| KeyError::InvalidMnemonic(e.to_string()))?;
        Self::from_mnemonic(&mnemonic.to_string())
    }

    /// Derive OGBank keys from an existing BIP-39 mnemonic phrase.
    pub fn from_mnemonic(mnemonic_str: &str) -> Result<Self, KeyError> {
        let mnemonic: bip39::Mnemonic = mnemonic_str
            .parse()
            .map_err(|e: bip39::Error| KeyError::InvalidMnemonic(e.to_string()))?;

        let seed = mnemonic.to_seed("");
        Self::from_seed_with_mnemonic(&seed, mnemonic_str.to_string())
    }

    /// Derive OGBank keys from a raw 64-byte seed.
    pub fn from_seed(seed: &[u8]) -> Result<Self, KeyError> {
        if seed.len() != 64 {
            return Err(KeyError::InvalidSeedLength(seed.len()));
        }
        Self::from_seed_with_mnemonic(seed, String::new())
    }

    fn from_seed_with_mnemonic(seed: &[u8], mnemonic: String) -> Result<Self, KeyError> {
        let account_id = AccountId::try_from(OGBANK_ACCOUNT_INDEX)
            .map_err(|e| KeyError::DerivationFailed(format!("invalid account index: {e}")))?;

        let sk = SpendingKey::from_zip32_seed(seed, ZCASH_COIN_TYPE, account_id)
            .map_err(|e| KeyError::DerivationFailed(format!("ZIP-32 derivation: {e:?}")))?;

        let fvk = FullViewingKey::from(&sk);
        let ivk = fvk.to_ivk(Scope::External);
        let ovk = fvk.to_ovk(Scope::External);
        let address = fvk.address_at(DiversifierIndex::new(), Scope::External);

        Ok(OGBankKeys {
            mnemonic,
            sk,
            fvk,
            ivk,
            ovk,
            address,
        })
    }

    /// Serialize the spending key bytes. Handle with extreme care.
    pub fn sk_bytes(&self) -> &[u8; 32] {
        self.sk.to_bytes()
    }

    /// Serialize the full viewing key to 96 bytes.
    pub fn fvk_bytes(&self) -> [u8; 96] {
        self.fvk.to_bytes()
    }

    /// Serialize the incoming viewing key to 64 bytes.
    pub fn ivk_bytes(&self) -> [u8; 64] {
        self.ivk.to_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // BIP-39 test vector: 24 words, deterministic. NEVER use in production.
    const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

    #[test]
    fn test_generate_mnemonic() {
        let keys = OGBankKeys::generate().expect("key generation failed");
        let words: Vec<&str> = keys.mnemonic.split_whitespace().collect();
        assert_eq!(words.len(), 24, "mnemonic must be 24 words");

        // Verify it's a valid BIP-39 mnemonic by re-parsing
        let _parsed: bip39::Mnemonic = keys.mnemonic.parse().expect("mnemonic should be valid");
    }

    #[test]
    fn test_seed_from_mnemonic() {
        let mnemonic: bip39::Mnemonic = TEST_MNEMONIC.parse().expect("test mnemonic invalid");
        let seed = mnemonic.to_seed("");
        assert_eq!(seed.len(), 64, "BIP-39 seed must be 64 bytes");
    }

    #[test]
    fn test_spending_key_from_seed() {
        let keys = OGBankKeys::from_mnemonic(TEST_MNEMONIC).expect("derivation failed");
        let sk_bytes = keys.sk_bytes();
        assert_eq!(sk_bytes.len(), 32, "spending key must be 32 bytes");
        assert!(sk_bytes.iter().any(|&b| b != 0), "sk must not be zero");
    }

    #[test]
    fn test_full_viewing_key_from_sk() {
        let keys = OGBankKeys::from_mnemonic(TEST_MNEMONIC).expect("derivation failed");
        let fvk_bytes = keys.fvk_bytes();
        assert_eq!(fvk_bytes.len(), 96, "fvk must be 96 bytes (ak + nk + rivk)");
        assert!(fvk_bytes.iter().any(|&b| b != 0), "fvk must not be zero");
    }

    #[test]
    fn test_incoming_viewing_key_from_fvk() {
        let keys = OGBankKeys::from_mnemonic(TEST_MNEMONIC).expect("derivation failed");
        let ivk_bytes = keys.ivk_bytes();
        assert_eq!(ivk_bytes.len(), 64, "ivk must be 64 bytes");
        assert!(ivk_bytes.iter().any(|&b| b != 0), "ivk must not be zero");
    }

    #[test]
    fn test_address_from_fvk() {
        let keys = OGBankKeys::from_mnemonic(TEST_MNEMONIC).expect("derivation failed");
        let addr_bytes = keys.address.to_raw_address_bytes();
        assert_eq!(
            addr_bytes.len(),
            43,
            "Orchard address must be 43 bytes (11 diversifier + 32 pk_d)"
        );
        assert!(addr_bytes.iter().any(|&b| b != 0), "address must not be zero");
    }

    #[test]
    fn test_deterministic_derivation() {
        let keys1 = OGBankKeys::from_mnemonic(TEST_MNEMONIC).expect("derivation 1 failed");
        let keys2 = OGBankKeys::from_mnemonic(TEST_MNEMONIC).expect("derivation 2 failed");

        assert_eq!(keys1.sk_bytes(), keys2.sk_bytes(), "sk must be deterministic");
        assert_eq!(keys1.fvk_bytes(), keys2.fvk_bytes(), "fvk must be deterministic");
        assert_eq!(keys1.ivk_bytes(), keys2.ivk_bytes(), "ivk must be deterministic");
        assert_eq!(
            keys1.address.to_raw_address_bytes(),
            keys2.address.to_raw_address_bytes(),
            "address must be deterministic"
        );
    }

    #[test]
    fn test_ogbank_path_isolation() {
        // OGBank uses account index 0x4F47 (20295), standard wallets use 0.
        let mnemonic: bip39::Mnemonic = TEST_MNEMONIC.parse().unwrap();
        let seed = mnemonic.to_seed("");

        // OGBank path: m/32'/133'/20295'
        let ogbank_keys = OGBankKeys::from_seed(&seed).expect("OGBank derivation failed");

        // Standard path: m/32'/133'/0'
        let standard_account = AccountId::try_from(0u32).unwrap();
        let standard_sk =
            SpendingKey::from_zip32_seed(&seed, ZCASH_COIN_TYPE, standard_account)
                .expect("standard derivation failed");

        assert_ne!(
            ogbank_keys.sk_bytes(),
            standard_sk.to_bytes(),
            "OGBank path (0x4F47) must differ from standard path (0)"
        );
    }

    #[test]
    fn test_1_to_1_binding() {
        // Different seeds produce different addresses.
        let keys1 = OGBankKeys::from_mnemonic(TEST_MNEMONIC).expect("derivation failed");

        let different_mnemonic = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo vote";
        let keys2 =
            OGBankKeys::from_mnemonic(different_mnemonic).expect("derivation 2 failed");

        assert_ne!(
            keys1.address.to_raw_address_bytes(),
            keys2.address.to_raw_address_bytes(),
            "different mnemonics must produce different addresses"
        );
    }

    #[test]
    fn test_invalid_mnemonic_rejected() {
        let result = OGBankKeys::from_mnemonic("not a valid mnemonic");
        assert!(result.is_err(), "invalid mnemonic must be rejected");
    }

    #[test]
    fn test_invalid_seed_length_rejected() {
        let short_seed = vec![0u8; 32];
        let result = OGBankKeys::from_seed(&short_seed);
        assert!(result.is_err(), "short seed must be rejected");

        match result.unwrap_err() {
            KeyError::InvalidSeedLength(len) => assert_eq!(len, 32),
            other => panic!("expected InvalidSeedLength, got: {other}"),
        }
    }

    #[test]
    fn test_fvk_roundtrip_serialization() {
        let keys = OGBankKeys::from_mnemonic(TEST_MNEMONIC).expect("derivation failed");
        let fvk_bytes = keys.fvk_bytes();
        let restored =
            FullViewingKey::from_bytes(&fvk_bytes).expect("fvk deserialization failed");
        assert_eq!(
            restored.to_bytes(),
            fvk_bytes,
            "fvk must roundtrip through serialization"
        );
    }
}

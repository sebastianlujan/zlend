use serde::{Deserialize, Serialize};
use thiserror::Error;
use zcash_keys::keys::UnifiedSpendingKey;
use zcash_protocol::consensus;
use zip32::AccountId;
use zeroize::Zeroize;

#[derive(Debug, Error)]
pub enum KeyError {
    #[error("mnemonic generation failed: {0}")]
    MnemonicError(String),
    #[error("key derivation failed: {0}")]
    DerivationError(String),
    #[error("address generation failed: {0}")]
    AddressError(String),
    #[error("serialization error: {0}")]
    SerializationError(String),
}

/// The full set of ZLend keys derived from a single seed via ZIP-32.
///
/// For MVP, all keys are hex-encoded for JSON transport.
/// In production, `sk` would be split via FROST 2-of-3.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZLendKeys {
    /// BIP-39 mnemonic (24 words) — only stored client-side
    pub mnemonic: String,
    /// Hex-encoded spending key bytes
    pub sk: String,
    /// Hex-encoded full viewing key bytes
    pub fvk: String,
    /// Hex-encoded incoming viewing key (for trial decryption)
    pub ivk: String,
    /// The ZLend shielded address (Zcash unified or Orchard address)
    pub address: String,
}

impl Drop for ZLendKeys {
    fn drop(&mut self) {
        self.sk.zeroize();
        self.mnemonic.zeroize();
    }
}

/// Generate a fresh 24-word BIP-39 mnemonic and derive ZLend keys.
///
/// Uses ZIP-32 Orchard key derivation at path `m/32'/133'/0'`
/// (coin_type 133 = Zcash, account 0).
pub fn generate_keys(network: &consensus::Network) -> Result<ZLendKeys, KeyError> {
    // Generate 256 bits of entropy for a 24-word mnemonic
    let mut entropy = [0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut entropy);

    let mnemonic = bip39::Mnemonic::from_entropy(&entropy)
        .map_err(|e| KeyError::MnemonicError(e.to_string()))?;

    derive_keys_from_mnemonic(&mnemonic.to_string(), network)
}

/// Derive ZLend keys from an existing mnemonic phrase.
///
/// This is deterministic — the same mnemonic always produces the same keys.
pub fn derive_keys_from_mnemonic(
    mnemonic_str: &str,
    network: &consensus::Network,
) -> Result<ZLendKeys, KeyError> {
    let mnemonic: bip39::Mnemonic = mnemonic_str
        .parse()
        .map_err(|e: bip39::Error| KeyError::MnemonicError(e.to_string()))?;

    // Derive seed from mnemonic (no passphrase for MVP)
    let seed = {
        let s = mnemonic.to_seed("");
        // Use first 32 bytes as the seed for ZIP-32
        let mut seed_bytes = [0u8; 32];
        seed_bytes.copy_from_slice(&s[..32]);
        seed_bytes
    };

    // Derive unified spending key at account 0
    let account_index = AccountId::try_from(0u32)
        .map_err(|e| KeyError::DerivationError(format!("invalid account index: {e}")))?;

    let usk = UnifiedSpendingKey::from_seed(network, &seed, account_index)
        .map_err(|e| KeyError::DerivationError(format!("USK derivation failed: {e}")))?;

    // Extract Orchard spending key
    let orchard_sk = usk
        .orchard()
        .to_bytes();
    let sk_hex = hex::encode(orchard_sk);

    // Derive full viewing key
    let orchard_fvk = orchard::keys::FullViewingKey::from(usk.orchard());
    let fvk_bytes = orchard_fvk.to_bytes();
    let fvk_hex = hex::encode(fvk_bytes);

    // Derive incoming viewing key (for trial decryption)
    let ivk = orchard_fvk.to_ivk(orchard::keys::Scope::External);
    let ivk_bytes = ivk.to_bytes();
    let ivk_hex = hex::encode(ivk_bytes.as_ref());

    // Generate default address (diversifier index 0, external scope)
    let address = orchard_fvk.address_at(
        zip32::DiversifierIndex::new(),
        orchard::keys::Scope::External,
    );
    let address_bytes = address.to_raw_address_bytes();
    let address_hex = hex::encode(address_bytes);

    Ok(ZLendKeys {
        mnemonic: mnemonic_str.to_string(),
        sk: sk_hex,
        fvk: fvk_hex,
        ivk: ivk_hex,
        address: address_hex,
    })
}

/// Network helper: returns the Zcash testnet configuration.
pub fn testnet() -> consensus::Network {
    consensus::Network::TestNetwork
}

/// Network helper: returns the Zcash mainnet configuration.
pub fn mainnet() -> consensus::Network {
    consensus::Network::MainNetwork
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_derivation() {
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
        let network = testnet();

        let keys1 = derive_keys_from_mnemonic(mnemonic, &network).unwrap();
        let keys2 = derive_keys_from_mnemonic(mnemonic, &network).unwrap();

        assert_eq!(keys1.sk, keys2.sk, "Same mnemonic should produce same sk");
        assert_eq!(keys1.fvk, keys2.fvk, "Same mnemonic should produce same fvk");
        assert_eq!(keys1.ivk, keys2.ivk, "Same mnemonic should produce same ivk");
        assert_eq!(keys1.address, keys2.address, "Same mnemonic should produce same address");
    }

    #[test]
    fn test_different_mnemonics_produce_different_keys() {
        let network = testnet();

        let keys1 = generate_keys(&network).unwrap();
        let keys2 = generate_keys(&network).unwrap();

        assert_ne!(keys1.sk, keys2.sk, "Different mnemonics should produce different keys");
    }

    #[test]
    fn test_keys_are_non_empty() {
        let network = testnet();
        let keys = generate_keys(&network).unwrap();

        assert!(!keys.sk.is_empty());
        assert!(!keys.fvk.is_empty());
        assert!(!keys.ivk.is_empty());
        assert!(!keys.address.is_empty());
        assert!(!keys.mnemonic.is_empty());

        // Mnemonic should be 24 words
        assert_eq!(keys.mnemonic.split_whitespace().count(), 24);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let network = testnet();
        let keys = generate_keys(&network).unwrap();

        let json = serde_json::to_string_pretty(&keys).unwrap();
        let deserialized: ZLendKeys = serde_json::from_str(&json).unwrap();

        assert_eq!(keys.sk, deserialized.sk);
        assert_eq!(keys.fvk, deserialized.fvk);
        assert_eq!(keys.ivk, deserialized.ivk);
        assert_eq!(keys.address, deserialized.address);
    }
}

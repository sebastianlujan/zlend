use orchard::{
    keys::{FullViewingKey, PreparedIncomingViewingKey, Scope},
    note::{ExtractedNoteCommitment, Nullifier},
    note_encryption::{CompactAction, OrchardDomain},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use zcash_note_encryption::{try_compact_note_decryption, EphemeralKeyBytes};

#[derive(Debug, Error)]
pub enum ScanError {
    #[error("invalid FVK bytes: {0}")]
    InvalidIvk(String),
    #[error("invalid encrypted data: {0}")]
    InvalidData(String),
    #[error("decryption failed")]
    DecryptionFailed,
}

/// A successfully decrypted Zcash note.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecryptedNote {
    /// Note value in zatoshis (1 ZEC = 100_000_000 zatoshis)
    pub value_zat: u64,
    /// Decrypted memo field (hex-encoded; empty for compact decryption)
    pub memo: String,
}

/// An Orchard action extracted from a Zcash transaction.
///
/// These fields come from the Tatum API (or any Zcash node RPC).
/// Each Orchard action in a transaction spends one note and creates one note.
/// The encrypted fields relate to the *created* note.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchardAction {
    /// Nullifier of the spent note (hex, 32 bytes)
    pub nf: String,
    /// Note commitment of the created note (hex, 32 bytes)
    pub cmx: String,
    /// Ephemeral public key for note encryption (hex, 32 bytes)
    #[serde(alias = "ephemeralKey")]
    pub ephemeral_key: String,
    /// Encrypted note ciphertext (hex, at least 52 bytes for compact decryption)
    #[serde(alias = "encCiphertext")]
    pub enc_ciphertext: String,
}

/// Attempt to decrypt an Orchard action using a full viewing key.
///
/// This is the core of ZLend's deposit verification. The relayer fetches
/// a transaction from the Zcash network, extracts Orchard actions, and
/// attempts trial decryption with the position's FVK (from which the IVK
/// is derived).
///
/// Uses **compact trial decryption** (first 52 bytes of ciphertext), which
/// recovers the note value and address but not the full memo. This is
/// sufficient for collateral verification.
///
/// # Arguments
///
/// * `fvk_bytes` - The 96-byte full viewing key (from `FullViewingKey::to_bytes()`)
/// * `action` - An Orchard action with hex-encoded fields
///
/// # Returns
///
/// `Ok(Some(DecryptedNote))` if the note belongs to this FVK, `Ok(None)` otherwise.
pub fn try_decrypt_action(
    fvk_bytes: &[u8],
    action: &OrchardAction,
) -> Result<Option<DecryptedNote>, ScanError> {
    // 1. Reconstruct FVK from 96 bytes
    if fvk_bytes.len() != 96 {
        return Err(ScanError::InvalidIvk(format!(
            "expected 96 bytes for FVK, got {}",
            fvk_bytes.len()
        )));
    }
    let fvk_array: [u8; 96] = fvk_bytes.try_into().unwrap();
    let fvk = FullViewingKey::from_bytes(&fvk_array)
        .ok_or_else(|| ScanError::InvalidIvk("invalid FVK bytes".into()))?;

    // 2. Derive and prepare IVK for trial decryption
    let ivk = fvk.to_ivk(Scope::External);
    let prepared_ivk = PreparedIncomingViewingKey::new(&ivk);

    // 3. Parse action fields from hex
    let nf_bytes: [u8; 32] = hex::decode(&action.nf)
        .map_err(|e| ScanError::InvalidData(format!("invalid nf hex: {}", e)))?
        .try_into()
        .map_err(|_| ScanError::InvalidData("nf must be 32 bytes".into()))?;

    let cmx_bytes: [u8; 32] = hex::decode(&action.cmx)
        .map_err(|e| ScanError::InvalidData(format!("invalid cmx hex: {}", e)))?
        .try_into()
        .map_err(|_| ScanError::InvalidData("cmx must be 32 bytes".into()))?;

    let epk_bytes: [u8; 32] = hex::decode(&action.ephemeral_key)
        .map_err(|e| ScanError::InvalidData(format!("invalid ephemeral_key hex: {}", e)))?
        .try_into()
        .map_err(|_| ScanError::InvalidData("ephemeral_key must be 32 bytes".into()))?;

    let enc_bytes = hex::decode(&action.enc_ciphertext)
        .map_err(|e| ScanError::InvalidData(format!("invalid enc_ciphertext hex: {}", e)))?;

    if enc_bytes.len() < 52 {
        return Err(ScanError::InvalidData(format!(
            "enc_ciphertext too short: need at least 52 bytes, got {}",
            enc_bytes.len()
        )));
    }

    // 4. Parse into Orchard types
    let nullifier = Option::from(Nullifier::from_bytes(&nf_bytes))
        .ok_or_else(|| ScanError::InvalidData("invalid nullifier bytes".into()))?;

    let cmx = Option::from(ExtractedNoteCommitment::from_bytes(&cmx_bytes))
        .ok_or_else(|| ScanError::InvalidData("invalid commitment bytes".into()))?;

    let compact_enc: [u8; 52] = enc_bytes[..52].try_into().unwrap();

    // 5. Build CompactAction and encryption domain
    let compact_action =
        CompactAction::from_parts(nullifier, cmx, EphemeralKeyBytes(epk_bytes), compact_enc);
    let domain = OrchardDomain::for_compact_action(&compact_action);

    // 6. Attempt compact trial decryption
    match try_compact_note_decryption(&domain, &prepared_ivk, &compact_action) {
        Some((note, _recipient)) => Ok(Some(DecryptedNote {
            value_zat: note.value().inner(),
            memo: String::new(), // compact decryption doesn't recover the memo
        })),
        None => Ok(None),
    }
}

/// Attempt to decrypt an Orchard note output using raw byte fields.
///
/// **Stub** — returns `Ok(None)` always. Kept for backward compatibility.
/// Use [`try_decrypt_action`] for real trial decryption.
pub fn try_decrypt_note(
    _ivk_bytes: &[u8],
    _encrypted_output: &[u8],
    _ephemeral_key: &[u8],
    _cmx: &[u8],
) -> Result<Option<DecryptedNote>, ScanError> {
    Ok(None)
}

/// Simulated trial decryption for testing and development.
///
/// This allows the relayer to process deposits without actual Zcash
/// cryptographic operations, using a simple value lookup.
pub fn try_decrypt_note_simulated(value_zat: u64, memo: &str) -> DecryptedNote {
    DecryptedNote {
        value_zat,
        memo: hex::encode(memo.as_bytes()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulated_decryption() {
        let note = try_decrypt_note_simulated(150_000_000, "zlend-deposit");

        assert_eq!(note.value_zat, 150_000_000);
        assert_eq!(note.memo, hex::encode(b"zlend-deposit"));
    }

    #[test]
    fn test_try_decrypt_returns_none_for_now() {
        let result = try_decrypt_note(&[0u8; 32], &[0u8; 64], &[0u8; 32], &[0u8; 32]);
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_try_decrypt_action_invalid_fvk_length() {
        let action = OrchardAction {
            nf: hex::encode([0u8; 32]),
            cmx: hex::encode([0u8; 32]),
            ephemeral_key: hex::encode([0u8; 32]),
            enc_ciphertext: hex::encode([0u8; 52]),
        };
        let result = try_decrypt_action(&[0u8; 32], &action); // 32 bytes, need 96
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("expected 96 bytes"));
    }

    #[test]
    fn test_try_decrypt_action_short_ciphertext() {
        let action = OrchardAction {
            nf: hex::encode([0u8; 32]),
            cmx: hex::encode([0u8; 32]),
            ephemeral_key: hex::encode([0u8; 32]),
            enc_ciphertext: hex::encode([0u8; 10]), // too short
        };
        let result = try_decrypt_action(&[0u8; 96], &action);
        // Either invalid FVK or short ciphertext — both are errors
        assert!(result.is_err());
    }

    #[test]
    fn test_orchard_action_serde_roundtrip() {
        let action = OrchardAction {
            nf: hex::encode([1u8; 32]),
            cmx: hex::encode([2u8; 32]),
            ephemeral_key: hex::encode([3u8; 32]),
            enc_ciphertext: hex::encode([4u8; 580]),
        };
        let json = serde_json::to_string(&action).unwrap();
        let decoded: OrchardAction = serde_json::from_str(&json).unwrap();
        assert_eq!(action.nf, decoded.nf);
        assert_eq!(action.cmx, decoded.cmx);
        assert_eq!(action.ephemeral_key, decoded.ephemeral_key);
        assert_eq!(action.enc_ciphertext, decoded.enc_ciphertext);
    }
}

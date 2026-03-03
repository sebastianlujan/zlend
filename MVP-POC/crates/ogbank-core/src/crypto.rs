// Phase 2: SK Encryption at Rest
//
// ChaCha20-Poly1305 with Argon2id key derivation.
// Storage format: salt(16) || nonce(12) || ciphertext
//
// Reference: docs/technical/08_mvp.md §SK Security

use argon2::Argon2;
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use rand::RngCore;
use zeroize::Zeroizing;

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

// Argon2id parameters: m=64MB, t=3 iterations, p=4 parallelism
const ARGON2_M_COST: u32 = 65536; // 64 MiB
const ARGON2_T_COST: u32 = 3;
const ARGON2_P_COST: u32 = 4;

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("invalid ciphertext: expected at least {0} bytes header, got {1}")]
    InvalidCiphertext(usize, usize),

    #[error("key derivation failed: {0}")]
    KeyDerivationFailed(String),
}

/// Derive a 256-bit symmetric key from a passphrase using Argon2id.
fn derive_key(passphrase: &str, salt: &[u8]) -> Result<Zeroizing<[u8; KEY_LEN]>, CryptoError> {
    let params = argon2::Params::new(ARGON2_M_COST, ARGON2_T_COST, ARGON2_P_COST, Some(KEY_LEN))
        .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

    let mut key = Zeroizing::new([0u8; KEY_LEN]);
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, key.as_mut())
        .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

    Ok(key)
}

/// Encrypt spending key bytes with ChaCha20-Poly1305.
///
/// Output format: `salt(16) || nonce(12) || ciphertext`
///
/// The encryption key is derived from the passphrase via Argon2id.
/// A random salt and nonce are generated for each call.
pub fn encrypt_sk(sk_bytes: &[u8], passphrase: &str) -> Result<Vec<u8>, CryptoError> {
    let mut salt = [0u8; SALT_LEN];
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut nonce_bytes);

    let key = derive_key(passphrase, &salt)?;
    let cipher = ChaCha20Poly1305::new_from_slice(key.as_ref())
        .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, sk_bytes)
        .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

    // salt(16) || nonce(12) || ciphertext
    let mut output = Vec::with_capacity(SALT_LEN + NONCE_LEN + ciphertext.len());
    output.extend_from_slice(&salt);
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);

    Ok(output)
}

/// Decrypt spending key bytes from the storage format.
///
/// Input format: `salt(16) || nonce(12) || ciphertext`
///
/// Returns the plaintext in a `Zeroizing` wrapper that clears memory on drop.
pub fn decrypt_sk(encrypted: &[u8], passphrase: &str) -> Result<Zeroizing<Vec<u8>>, CryptoError> {
    let header_len = SALT_LEN + NONCE_LEN;
    if encrypted.len() < header_len {
        return Err(CryptoError::InvalidCiphertext(header_len, encrypted.len()));
    }

    let salt = &encrypted[..SALT_LEN];
    let nonce_bytes = &encrypted[SALT_LEN..header_len];
    let ciphertext = &encrypted[header_len..];

    let key = derive_key(passphrase, salt)?;
    let cipher = ChaCha20Poly1305::new_from_slice(key.as_ref())
        .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;

    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;

    Ok(Zeroizing::new(plaintext))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PASSPHRASE: &str = "test-passphrase-for-ogbank";
    const TEST_SK: [u8; 32] = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
        0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c,
        0x1d, 0x1e, 0x1f, 0x20,
    ];

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let encrypted = encrypt_sk(&TEST_SK, TEST_PASSPHRASE).expect("encryption failed");
        let decrypted = decrypt_sk(&encrypted, TEST_PASSPHRASE).expect("decryption failed");
        assert_eq!(decrypted.as_slice(), &TEST_SK, "roundtrip must preserve data");
    }

    #[test]
    fn test_wrong_passphrase_fails() {
        let encrypted = encrypt_sk(&TEST_SK, TEST_PASSPHRASE).expect("encryption failed");
        let result = decrypt_sk(&encrypted, "wrong-passphrase");
        assert!(result.is_err(), "wrong passphrase must fail decryption");

        match result.unwrap_err() {
            CryptoError::DecryptionFailed(_) => {}
            other => panic!("expected DecryptionFailed, got: {other}"),
        }
    }

    #[test]
    fn test_different_salt_different_output() {
        let encrypted1 = encrypt_sk(&TEST_SK, TEST_PASSPHRASE).expect("encryption 1 failed");
        let encrypted2 = encrypt_sk(&TEST_SK, TEST_PASSPHRASE).expect("encryption 2 failed");

        // Salt is random, so outputs must differ
        assert_ne!(
            encrypted1, encrypted2,
            "same plaintext + same passphrase must produce different ciphertext (random salt)"
        );

        // But both must decrypt to the same value
        let d1 = decrypt_sk(&encrypted1, TEST_PASSPHRASE).unwrap();
        let d2 = decrypt_sk(&encrypted2, TEST_PASSPHRASE).unwrap();
        assert_eq!(d1.as_slice(), d2.as_slice(), "both must decrypt to same plaintext");
    }

    #[test]
    fn test_storage_format() {
        let encrypted = encrypt_sk(&TEST_SK, TEST_PASSPHRASE).expect("encryption failed");

        // salt(16) + nonce(12) + ciphertext(32 plaintext + 16 tag)
        let expected_len = SALT_LEN + NONCE_LEN + TEST_SK.len() + 16; // Poly1305 tag is 16 bytes
        assert_eq!(
            encrypted.len(),
            expected_len,
            "format must be salt(16) + nonce(12) + ciphertext(plaintext + 16 tag)"
        );

        // Salt is first 16 bytes
        let salt = &encrypted[..SALT_LEN];
        assert!(salt.iter().any(|&b| b != 0), "salt must not be zero");

        // Nonce is next 12 bytes
        let nonce = &encrypted[SALT_LEN..SALT_LEN + NONCE_LEN];
        assert_eq!(nonce.len(), NONCE_LEN);
    }

    #[test]
    fn test_argon2id_params() {
        // Verify our constants match the documented params
        assert_eq!(ARGON2_M_COST, 65536, "m_cost must be 64MB (65536 KiB)");
        assert_eq!(ARGON2_T_COST, 3, "t_cost must be 3 iterations");
        assert_eq!(ARGON2_P_COST, 4, "p_cost must be 4 parallelism");
    }

    #[test]
    fn test_invalid_ciphertext_rejected() {
        // Too short to contain header
        let short = vec![0u8; 10];
        let result = decrypt_sk(&short, TEST_PASSPHRASE);
        assert!(result.is_err());

        match result.unwrap_err() {
            CryptoError::InvalidCiphertext(expected, actual) => {
                assert_eq!(expected, SALT_LEN + NONCE_LEN);
                assert_eq!(actual, 10);
            }
            other => panic!("expected InvalidCiphertext, got: {other}"),
        }
    }

    #[test]
    fn test_empty_plaintext() {
        // Edge case: encrypting empty data should work
        let encrypted = encrypt_sk(&[], TEST_PASSPHRASE).expect("encryption failed");
        let decrypted = decrypt_sk(&encrypted, TEST_PASSPHRASE).expect("decryption failed");
        assert!(decrypted.is_empty(), "decrypted empty data must be empty");
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        let mut encrypted = encrypt_sk(&TEST_SK, TEST_PASSPHRASE).expect("encryption failed");

        // Flip a bit in the ciphertext portion
        let last = encrypted.len() - 1;
        encrypted[last] ^= 0x01;

        let result = decrypt_sk(&encrypted, TEST_PASSPHRASE);
        assert!(result.is_err(), "tampered ciphertext must fail AEAD verification");
    }
}

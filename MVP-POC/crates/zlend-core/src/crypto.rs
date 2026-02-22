use chacha20poly1305::{
    aead::{Aead, OsRng},
    AeadCore, ChaCha20Poly1305, KeyInit,
};
use thiserror::Error;
use zeroize::Zeroize;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("invalid encrypted data length: expected at least 44 bytes, got {0}")]
    InvalidLength(usize),
    #[error("key derivation failed: {0}")]
    KeyDerivationFailed(String),
}

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const HEADER_LEN: usize = SALT_LEN + NONCE_LEN; // 28 bytes before ciphertext

/// Derive a 256-bit encryption key from a passphrase + salt using Argon2id.
///
/// Uses conservative parameters suitable for server-side key derivation:
/// - m_cost: 19456 KiB (~19 MB)
/// - t_cost: 2 iterations
/// - p_cost: 1 lane
pub fn derive_key(passphrase: &[u8], salt: &[u8; SALT_LEN]) -> Result<[u8; 32], CryptoError> {
    let params = argon2::Params::new(19456, 2, 1, Some(32))
        .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;
    let argon2 = argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

    let mut key = [0u8; 32];
    argon2
        .hash_password_into(passphrase, salt, &mut key)
        .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

    Ok(key)
}

/// Encrypt spending key bytes using ChaCha20-Poly1305.
///
/// Returns: `salt(16) || nonce(12) || ciphertext(len + 16 tag)`
///
/// A fresh random salt and nonce are generated for each call,
/// so encrypting the same sk twice produces different output.
pub fn encrypt_sk(sk: &[u8], passphrase: &str) -> Result<Vec<u8>, CryptoError> {
    use rand::RngCore;

    // Generate random salt
    let mut salt = [0u8; SALT_LEN];
    rand::thread_rng().fill_bytes(&mut salt);

    // Derive key from passphrase
    let mut key = derive_key(passphrase.as_bytes(), &salt)?;
    let cipher = ChaCha20Poly1305::new_from_slice(&key)
        .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

    // Zero the key material as soon as the cipher is constructed
    key.zeroize();

    // Generate random nonce
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);

    // Encrypt
    let ciphertext = cipher
        .encrypt(&nonce, sk)
        .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

    // Pack: salt || nonce || ciphertext
    let mut result = Vec::with_capacity(HEADER_LEN + ciphertext.len());
    result.extend_from_slice(&salt);
    result.extend_from_slice(&nonce);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

/// Decrypt spending key bytes from the format produced by [`encrypt_sk`].
///
/// Input format: `salt(16) || nonce(12) || ciphertext`
///
/// The caller should zeroize the returned `Vec<u8>` after use.
pub fn decrypt_sk(encrypted: &[u8], passphrase: &str) -> Result<Vec<u8>, CryptoError> {
    if encrypted.len() < HEADER_LEN + 16 {
        // 16 bytes minimum for the Poly1305 tag
        return Err(CryptoError::InvalidLength(encrypted.len()));
    }

    // Unpack: salt || nonce || ciphertext
    let salt: [u8; SALT_LEN] = encrypted[..SALT_LEN]
        .try_into()
        .map_err(|_| CryptoError::DecryptionFailed("invalid salt".into()))?;
    let nonce_bytes = &encrypted[SALT_LEN..HEADER_LEN];
    let ciphertext = &encrypted[HEADER_LEN..];

    // Derive key
    let mut key = derive_key(passphrase.as_bytes(), &salt)?;
    let cipher = ChaCha20Poly1305::new_from_slice(&key)
        .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;
    key.zeroize();

    let nonce = chacha20poly1305::Nonce::from_slice(nonce_bytes);

    // Decrypt
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| CryptoError::DecryptionFailed("wrong passphrase or corrupted data".into()))?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let sk = b"this-is-a-fake-spending-key-32by";
        let passphrase = "test-passphrase-for-mvp";

        let encrypted = encrypt_sk(sk, passphrase).unwrap();
        let decrypted = decrypt_sk(&encrypted, passphrase).unwrap();

        assert_eq!(&decrypted, sk);
    }

    #[test]
    fn test_different_encryptions_produce_different_output() {
        let sk = b"same-key-encrypted-twice-32byte";
        let passphrase = "same-passphrase";

        let enc1 = encrypt_sk(sk, passphrase).unwrap();
        let enc2 = encrypt_sk(sk, passphrase).unwrap();

        // Different salt + nonce → different ciphertext
        assert_ne!(enc1, enc2);

        // But both decrypt to the same value
        assert_eq!(decrypt_sk(&enc1, passphrase).unwrap(), sk);
        assert_eq!(decrypt_sk(&enc2, passphrase).unwrap(), sk);
    }

    #[test]
    fn test_wrong_passphrase_fails() {
        let sk = b"another-fake-spending-key-32byte";
        let encrypted = encrypt_sk(sk, "correct-passphrase").unwrap();

        let result = decrypt_sk(&encrypted, "wrong-passphrase");
        assert!(result.is_err());
    }

    #[test]
    fn test_corrupted_data_fails() {
        let sk = b"yet-another-fake-sk-value-32byte";
        let mut encrypted = encrypt_sk(sk, "passphrase").unwrap();

        // Corrupt a byte in the ciphertext
        let last = encrypted.len() - 1;
        encrypted[last] ^= 0xFF;

        let result = decrypt_sk(&encrypted, "passphrase");
        assert!(result.is_err());
    }

    #[test]
    fn test_too_short_data_fails() {
        let result = decrypt_sk(&[0u8; 10], "passphrase");
        assert!(matches!(result, Err(CryptoError::InvalidLength(10))));
    }
}

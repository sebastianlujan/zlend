//! End-to-end tests for the ZLend Zcash crypto flow.
//!
//! Tests the full identity lifecycle:
//!   Phase 0a: Key generation (ZIP-32 derivation)
//!   Phase 0b: SK encryption at rest (ChaCha20-Poly1305 + Argon2)
//!   Phase 1:  Orchard trial decryption (compact note decryption)
//!   Phase 2:  Full crypto lifecycle (generate → encrypt → decrypt note)

use orchard::{
    keys::{FullViewingKey, Scope, SpendingKey},
    note::{ExtractedNoteCommitment, Nullifier, Rho},
    note_encryption::OrchardDomain,
    value::NoteValue,
    Note,
};
use zcash_note_encryption::{Domain, EphemeralKeyBytes, NoteEncryption};
use zcash_protocol::consensus;
use zip32::AccountId;

use zlend_core::{
    crypto,
    keys::{self, derive_keys_from_mnemonic, generate_keys},
    scan::{try_decrypt_action, OrchardAction},
};

/// A known-good test mnemonic (from BIP-39 test vectors).
const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

// ─── Phase 0a: Identity Generation ───

#[test]
fn test_phase0a_generate_identity_produces_valid_keys() {
    let network = keys::testnet();
    let keys = generate_keys(&network).unwrap();

    // Mnemonic is 24 words
    assert_eq!(keys.mnemonic.split_whitespace().count(), 24);

    // All hex fields are non-empty and valid hex
    assert!(!keys.sk.is_empty());
    assert!(!keys.fvk.is_empty());
    assert!(!keys.ivk.is_empty());
    assert!(!keys.address.is_empty());
    assert!(hex::decode(&keys.sk).is_ok());
    assert!(hex::decode(&keys.fvk).is_ok());
    assert!(hex::decode(&keys.ivk).is_ok());
    assert!(hex::decode(&keys.address).is_ok());

    // FVK is 96 bytes (Orchard full viewing key)
    assert_eq!(hex::decode(&keys.fvk).unwrap().len(), 96);
}

#[test]
fn test_phase0a_deterministic_derivation_same_mnemonic() {
    let network = keys::testnet();

    let keys1 = derive_keys_from_mnemonic(TEST_MNEMONIC, &network).unwrap();
    let keys2 = derive_keys_from_mnemonic(TEST_MNEMONIC, &network).unwrap();

    assert_eq!(keys1.sk, keys2.sk, "Same mnemonic → same sk");
    assert_eq!(keys1.fvk, keys2.fvk, "Same mnemonic → same fvk");
    assert_eq!(keys1.ivk, keys2.ivk, "Same mnemonic → same ivk");
    assert_eq!(keys1.address, keys2.address, "Same mnemonic → same address");
}

#[test]
fn test_phase0a_different_mnemonics_produce_different_identities() {
    let network = keys::testnet();

    let keys1 = generate_keys(&network).unwrap();
    let keys2 = generate_keys(&network).unwrap();

    assert_ne!(keys1.sk, keys2.sk);
    assert_ne!(keys1.fvk, keys2.fvk);
    assert_ne!(keys1.address, keys2.address);
}

// ─── Phase 0b: SK Encryption at Rest ───

#[test]
fn test_sk_encryption_roundtrip() {
    let network = keys::testnet();
    let keys = generate_keys(&network).unwrap();
    let sk_bytes = hex::decode(&keys.sk).unwrap();

    let passphrase = "zlend-test-passphrase-2024";
    let encrypted = crypto::encrypt_sk(&sk_bytes, passphrase).unwrap();
    let decrypted = crypto::decrypt_sk(&encrypted, passphrase).unwrap();

    assert_eq!(decrypted, sk_bytes, "Decrypted SK must match original");
}

#[test]
fn test_sk_encryption_wrong_passphrase_fails() {
    let sk = b"test-spending-key-32-bytes-long!";
    let encrypted = crypto::encrypt_sk(sk, "correct-passphrase").unwrap();
    let result = crypto::decrypt_sk(&encrypted, "wrong-passphrase");

    assert!(result.is_err(), "Wrong passphrase must fail decryption");
}

#[test]
fn test_sk_encryption_produces_different_ciphertexts() {
    let sk = b"same-key-encrypted-multiple-time";
    let passphrase = "same-passphrase";

    let enc1 = crypto::encrypt_sk(sk, passphrase).unwrap();
    let enc2 = crypto::encrypt_sk(sk, passphrase).unwrap();

    // Different random salt + nonce each time
    assert_ne!(enc1, enc2, "Same plaintext must produce different ciphertexts");

    // Both decrypt to the same value
    assert_eq!(crypto::decrypt_sk(&enc1, passphrase).unwrap(), sk);
    assert_eq!(crypto::decrypt_sk(&enc2, passphrase).unwrap(), sk);
}

// ─── Phase 1: Orchard Trial Decryption ───

/// Helper: Create a real encrypted Orchard note and return the fields
/// needed for trial decryption.
fn create_encrypted_note(
    fvk: &FullViewingKey,
    value_zat: u64,
) -> (OrchardAction, [u8; 96]) {
    // Derive the recipient address from the FVK
    let address = fvk.address_at(
        zip32::DiversifierIndex::new(),
        Scope::External,
    );

    // Create Rho and Nullifier from the same bytes (valid pallas field element)
    let mut rho_bytes = [0u8; 32];
    rho_bytes[0] = 1; // Small valid field element
    let rho: Rho = Option::from(Rho::from_bytes(&rho_bytes))
        .expect("Rho::from_bytes failed");
    let nullifier: Nullifier = Option::from(Nullifier::from_bytes(&rho_bytes))
        .expect("Nullifier::from_bytes failed");

    // Create RandomSeed
    let mut rseed_bytes = [0u8; 32];
    rseed_bytes[0] = 42;
    rseed_bytes[1] = 7;
    let rseed: orchard::note::RandomSeed = Option::from(orchard::note::RandomSeed::from_bytes(rseed_bytes, &rho))
        .expect("RandomSeed::from_bytes failed");

    // Create the note
    let value = NoteValue::from_raw(value_zat);
    let note: Note = Option::from(Note::from_parts(address, value, rho, rseed))
        .expect("Note::from_parts failed");

    // Extract commitment BEFORE passing note to NoteEncryption (which takes ownership)
    let cmx = ExtractedNoteCommitment::from(note.commitment());

    // Encrypt the note
    let ovk = fvk.to_ovk(Scope::External);
    let memo = [0u8; 512];
    let ne = NoteEncryption::<OrchardDomain>::new(Some(ovk), note, memo);

    // Extract ephemeral key bytes
    let epk_bytes: EphemeralKeyBytes = OrchardDomain::epk_bytes(ne.epk());

    // Extract encrypted ciphertext
    let enc_ciphertext = ne.encrypt_note_plaintext();

    // Build the OrchardAction (hex-encoded fields, as they'd come from Tatum API)
    let action = OrchardAction {
        nf: hex::encode(nullifier.to_bytes()),
        cmx: hex::encode(cmx.to_bytes()),
        ephemeral_key: hex::encode(epk_bytes.0),
        enc_ciphertext: hex::encode(enc_ciphertext.as_ref()),
    };

    (action, fvk.to_bytes())
}

/// Helper: Derive FVK from test mnemonic
fn test_fvk() -> FullViewingKey {
    let network = consensus::Network::TestNetwork;
    let mnemonic: bip39::Mnemonic = TEST_MNEMONIC.parse().unwrap();
    let seed = mnemonic.to_seed("");
    let mut seed_bytes = [0u8; 32];
    seed_bytes.copy_from_slice(&seed[..32]);

    let account_index = AccountId::try_from(0u32).unwrap();
    let usk = zcash_keys::keys::UnifiedSpendingKey::from_seed(&network, &seed_bytes, account_index)
        .unwrap();
    FullViewingKey::from(usk.orchard())
}

#[test]
fn test_trial_decryption_with_real_orchard_note() {
    let fvk = test_fvk();
    let deposit_value: u64 = 150_000_000; // 1.5 ZEC

    // Create a real encrypted Orchard note
    let (action, fvk_bytes) = create_encrypted_note(&fvk, deposit_value);

    // Attempt trial decryption with the correct FVK
    let result = try_decrypt_action(&fvk_bytes, &action).unwrap();

    assert!(result.is_some(), "Decryption must succeed with correct FVK");
    let decrypted = result.unwrap();
    assert_eq!(
        decrypted.value_zat, deposit_value,
        "Decrypted value must match the deposited amount"
    );
}

#[test]
fn test_trial_decryption_wrong_fvk_returns_none() {
    let fvk = test_fvk();
    let deposit_value: u64 = 100_000_000; // 1.0 ZEC

    // Create a note encrypted for the test FVK
    let (action, _correct_fvk_bytes) = create_encrypted_note(&fvk, deposit_value);

    // Generate a DIFFERENT FVK (from a different spending key)
    let wrong_sk_bytes = {
        let mut bytes = [0u8; 32];
        bytes[0] = 0xFF;
        bytes[1] = 0xAB;
        bytes[2] = 0xCD;
        bytes
    };
    let wrong_sk: SpendingKey = Option::from(SpendingKey::from_bytes(wrong_sk_bytes))
        .expect("SpendingKey::from_bytes failed");
    let wrong_fvk = FullViewingKey::from(&wrong_sk as &SpendingKey);
    let wrong_fvk_bytes = wrong_fvk.to_bytes();

    // Attempt trial decryption with the WRONG FVK
    let result = try_decrypt_action(&wrong_fvk_bytes, &action).unwrap();

    assert!(
        result.is_none(),
        "Decryption must fail with wrong FVK — note doesn't belong to this key"
    );
}

#[test]
fn test_trial_decryption_various_amounts() {
    let fvk = test_fvk();

    // Test with different deposit amounts
    let amounts: Vec<u64> = vec![
        1,                  // 1 zatoshi (minimum)
        100_000,            // 0.001 ZEC
        50_000_000,         // 0.5 ZEC
        100_000_000,        // 1.0 ZEC
        150_000_000,        // 1.5 ZEC (default collateral)
        1_000_000_000,      // 10 ZEC
        2_100_000_000_000_000, // 21M ZEC (max supply in zatoshis)
    ];

    for amount in amounts {
        let (action, fvk_bytes) = create_encrypted_note(&fvk, amount);
        let result = try_decrypt_action(&fvk_bytes, &action)
            .unwrap_or_else(|e| panic!("Decryption error for amount {}: {}", amount, e));

        assert!(result.is_some(), "Decryption must succeed for amount {}", amount);
        assert_eq!(
            result.unwrap().value_zat, amount,
            "Decrypted value must match for amount {}",
            amount
        );
    }
}

// ─── Phase 2: Full Crypto Lifecycle ───

#[test]
fn test_full_zcash_identity_lifecycle() {
    let network = keys::testnet();

    // Phase 0a: Generate identity
    let keys = generate_keys(&network).unwrap();
    assert_eq!(keys.mnemonic.split_whitespace().count(), 24);

    // Phase 0b: Encrypt SK at rest
    let sk_bytes = hex::decode(&keys.sk).unwrap();
    let passphrase = "relayer-test-passphrase";
    let sk_encrypted = crypto::encrypt_sk(&sk_bytes, passphrase).unwrap();

    // Verify encrypted SK is different from plaintext
    assert_ne!(sk_encrypted, sk_bytes);

    // Phase 0b: Decrypt SK (simulating relayer needing to spend)
    let sk_decrypted = crypto::decrypt_sk(&sk_encrypted, passphrase).unwrap();
    assert_eq!(sk_decrypted, sk_bytes, "Decrypted SK must match original");

    // Phase 1: Reconstruct FVK and verify trial decryption
    let fvk_bytes = hex::decode(&keys.fvk).unwrap();
    let fvk_array: [u8; 96] = fvk_bytes.clone().try_into().unwrap();
    let fvk = FullViewingKey::from_bytes(&fvk_array).unwrap();

    // Simulate a deposit: create an encrypted note
    let deposit_value: u64 = 200_000_000; // 2.0 ZEC
    let (action, _) = create_encrypted_note(&fvk, deposit_value);

    // Verify the deposit via trial decryption
    let result = try_decrypt_action(&fvk_bytes, &action).unwrap();
    assert!(result.is_some(), "Must detect deposit via trial decryption");
    assert_eq!(result.unwrap().value_zat, deposit_value);

    // Verify deterministic re-derivation
    let keys2 = derive_keys_from_mnemonic(&keys.mnemonic, &network).unwrap();
    assert_eq!(keys.fvk, keys2.fvk, "Re-derived FVK must match");
    assert_eq!(keys.address, keys2.address, "Re-derived address must match");

    // Re-derived FVK can also decrypt the same note
    let fvk_bytes2 = hex::decode(&keys2.fvk).unwrap();
    let result2 = try_decrypt_action(&fvk_bytes2, &action).unwrap();
    assert!(result2.is_some(), "Re-derived FVK must also decrypt the note");
    assert_eq!(result2.unwrap().value_zat, deposit_value);
}

// Phase 3: Trial Decryption
//
// Given ivk + encrypted Orchard actions → attempt decryption → recover (d, v, cmx)
//
// Compact decryption (no memo) for lightweight scanning.
// Full decryption (with memo) for complete note recovery.
//
// Reference: docs/technical/05_zcash-integration.md §Trial Decryption

use orchard::keys::{FullViewingKey, PreparedIncomingViewingKey};
use orchard::Action;
use orchard::note::{ExtractedNoteCommitment, Nullifier};
use orchard::note_encryption::{CompactAction, OrchardDomain};
use orchard::{Address, Note};
use zcash_note_encryption::{try_compact_note_decryption, try_note_decryption};

/// A note recovered via compact trial decryption (no memo).
#[derive(Debug, Clone)]
pub struct DecryptedNote {
    /// Note value in zatoshis.
    pub value_zat: u64,
    /// Recipient address (should match the OGBank address).
    pub recipient: Address,
    /// Extracted note commitment from the action.
    pub cmx: ExtractedNoteCommitment,
    /// The full decrypted note — needed to derive the nullifier later.
    note: Note,
}

impl DecryptedNote {
    /// Derive the nullifier for this note using the full viewing key.
    ///
    /// The nullifier is needed to detect when this note is spent.
    pub fn nullifier(&self, fvk: &FullViewingKey) -> Nullifier {
        self.note.nullifier(fvk)
    }
}

/// A note recovered via full trial decryption (includes 512-byte memo).
#[derive(Debug, Clone)]
pub struct DecryptedNoteWithMemo {
    /// Note value in zatoshis.
    pub value_zat: u64,
    /// Recipient address.
    pub recipient: Address,
    /// 512-byte memo field from the encrypted note.
    pub memo: [u8; 512],
    /// Extracted note commitment from the action.
    pub cmx: ExtractedNoteCommitment,
    /// The full decrypted note.
    note: Note,
}

impl DecryptedNoteWithMemo {
    /// Derive the nullifier for this note using the full viewing key.
    pub fn nullifier(&self, fvk: &FullViewingKey) -> Nullifier {
        self.note.nullifier(fvk)
    }
}

/// Attempt compact trial decryption of a single Orchard action.
///
/// Returns `Some(DecryptedNote)` if the note was encrypted to the given `ivk`,
/// or `None` if decryption fails (the note belongs to a different recipient).
pub fn try_decrypt_compact(
    ivk: &PreparedIncomingViewingKey,
    action: &CompactAction,
) -> Option<DecryptedNote> {
    let domain = OrchardDomain::for_compact_action(action);
    let (note, recipient) = try_compact_note_decryption(&domain, ivk, action)?;

    Some(DecryptedNote {
        value_zat: note.value().inner(),
        recipient,
        cmx: action.cmx(),
        note,
    })
}

/// Attempt full trial decryption of a single Orchard action (with memo).
///
/// Returns `Some(DecryptedNoteWithMemo)` if the note was encrypted to the given `ivk`,
/// or `None` if decryption fails.
pub fn try_decrypt_full<T>(
    ivk: &PreparedIncomingViewingKey,
    action: &Action<T>,
) -> Option<DecryptedNoteWithMemo> {
    let domain = OrchardDomain::for_action(action);
    let (note, recipient, memo) = try_note_decryption(&domain, ivk, action)?;

    Some(DecryptedNoteWithMemo {
        value_zat: note.value().inner(),
        recipient,
        memo,
        cmx: *action.cmx(),
        note,
    })
}

/// Scan a list of compact Orchard actions, returning all notes decryptable with the given ivk.
pub fn scan_compact_actions(
    ivk: &PreparedIncomingViewingKey,
    actions: &[CompactAction],
) -> Vec<DecryptedNote> {
    actions
        .iter()
        .filter_map(|action| try_decrypt_compact(ivk, action))
        .collect()
}

/// Scan a list of full Orchard actions, returning all decryptable notes with memos.
pub fn scan_actions<T>(
    ivk: &PreparedIncomingViewingKey,
    actions: &[Action<T>],
) -> Vec<DecryptedNoteWithMemo> {
    actions
        .iter()
        .filter_map(|action| try_decrypt_full(ivk, action))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::OGBankKeys;
    use orchard::keys::PreparedIncomingViewingKey;
    use orchard::note::{ExtractedNoteCommitment, Nullifier, RandomSeed, Rho};
    use orchard::note_encryption::OrchardNoteEncryption;
    use orchard::value::NoteValue;
    use rand::rngs::OsRng;
    use rand::RngCore;
    use zcash_note_encryption::Domain;

    const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
    const OTHER_MNEMONIC: &str = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo vote";

    fn test_keys() -> OGBankKeys {
        OGBankKeys::from_mnemonic(TEST_MNEMONIC).expect("test key derivation failed")
    }

    fn other_keys() -> OGBankKeys {
        OGBankKeys::from_mnemonic(OTHER_MNEMONIC).expect("other key derivation failed")
    }

    /// Create a test nullifier from a seed byte.
    fn test_nullifier(seed: u8) -> Nullifier {
        let mut bytes = [0u8; 32];
        bytes[0] = seed.max(1);
        Option::from(Nullifier::from_bytes(&bytes)).expect("test nullifier must be valid")
    }

    /// Create a fake CompactAction encrypted to the given address.
    ///
    /// Mirrors `orchard::note_encryption::testing::fake_compact_action` but uses
    /// only public APIs (avoids the `test-dependencies` feature which conflicts
    /// with alloy's proptest version).
    fn make_compact_action(
        address: Address,
        value_zat: u64,
        nf_seed: u8,
    ) -> (CompactAction, Note) {
        let nf = test_nullifier(nf_seed);

        // Rho is derived from the nullifier (same as CompactAction::rho() does internally)
        let rho = Option::from(Rho::from_bytes(&nf.to_bytes()))
            .expect("rho from nullifier bytes must be valid");

        // Generate a valid RandomSeed (must produce a non-zero esk)
        let rseed = loop {
            let mut bytes = [0u8; 32];
            OsRng.fill_bytes(&mut bytes);
            let candidate = RandomSeed::from_bytes(bytes, &rho);
            if bool::from(candidate.is_some()) {
                break Option::from(candidate).unwrap();
            }
        };

        // Build the note
        let note = Option::from(Note::from_parts(
            address,
            NoteValue::from_raw(value_zat),
            rho,
            rseed,
        ))
        .expect("note construction must succeed");

        // Encrypt the note
        let encryptor = OrchardNoteEncryption::new(None, note, [0u8; 512]);
        let cmx = ExtractedNoteCommitment::from(note.commitment());
        let ephemeral_key = OrchardDomain::epk_bytes(encryptor.epk());
        let enc_ciphertext = encryptor.encrypt_note_plaintext();

        let compact = CompactAction::from_parts(
            nf,
            cmx,
            ephemeral_key,
            enc_ciphertext[..52].try_into().unwrap(),
        );

        (compact, note)
    }

    #[test]
    fn test_decrypt_own_note() {
        let keys = test_keys();
        let pivk = PreparedIncomingViewingKey::new(&keys.ivk);

        let (compact, _note) = make_compact_action(keys.address, 1_000_000, 1);

        let result = try_decrypt_compact(&pivk, &compact);
        assert!(result.is_some(), "must decrypt note sent to our address");

        let decrypted = result.unwrap();
        assert_eq!(decrypted.value_zat, 1_000_000, "value must be 0.01 ZEC");
    }

    #[test]
    fn test_decrypt_foreign_note_fails() {
        let keys = test_keys();
        let pivk = PreparedIncomingViewingKey::new(&keys.ivk);

        // Encrypt to a DIFFERENT address
        let other = other_keys();
        let (compact, _) = make_compact_action(other.address, 500_000, 2);

        let result = try_decrypt_compact(&pivk, &compact);
        assert!(
            result.is_none(),
            "must not decrypt note for different address"
        );
    }

    #[test]
    fn test_scan_multiple_actions() {
        let keys = test_keys();
        let pivk = PreparedIncomingViewingKey::new(&keys.ivk);
        let other = other_keys();

        // 5 actions: only indices 1 and 3 are ours
        let (a0, _) = make_compact_action(other.address, 100_000, 10);
        let (a1, _) = make_compact_action(keys.address, 200_000, 11);
        let (a2, _) = make_compact_action(other.address, 300_000, 12);
        let (a3, _) = make_compact_action(keys.address, 400_000, 13);
        let (a4, _) = make_compact_action(other.address, 500_000, 14);

        let actions = vec![a0, a1, a2, a3, a4];
        let found = scan_compact_actions(&pivk, &actions);

        assert_eq!(found.len(), 2, "must find exactly 2 notes for our address");
        assert_eq!(found[0].value_zat, 200_000);
        assert_eq!(found[1].value_zat, 400_000);
    }

    #[test]
    fn test_value_extraction() {
        let keys = test_keys();
        let pivk = PreparedIncomingViewingKey::new(&keys.ivk);

        let test_values: &[u64] = &[1, 100_000_000, 50_000, 1_500_000_000];

        for (i, &value) in test_values.iter().enumerate() {
            let (compact, _) = make_compact_action(keys.address, value, (20 + i) as u8);
            let decrypted =
                try_decrypt_compact(&pivk, &compact).expect("decryption must succeed");
            assert_eq!(
                decrypted.value_zat, value,
                "value must match for {value} zatoshis"
            );
        }
    }

    #[test]
    fn test_commitment_matches_action() {
        let keys = test_keys();
        let pivk = PreparedIncomingViewingKey::new(&keys.ivk);

        let (compact, _note) = make_compact_action(keys.address, 1_000_000, 30);
        let decrypted = try_decrypt_compact(&pivk, &compact).unwrap();

        assert_eq!(
            decrypted.cmx.to_bytes(),
            compact.cmx().to_bytes(),
            "decrypted cmx must match the action's cmx"
        );
    }

    #[test]
    fn test_nullifier_derivation() {
        let keys = test_keys();
        let pivk = PreparedIncomingViewingKey::new(&keys.ivk);

        let (compact, _) = make_compact_action(keys.address, 1_000_000, 40);
        let decrypted = try_decrypt_compact(&pivk, &compact).unwrap();

        // Derive the nullifier using the full viewing key
        let nullifier = decrypted.nullifier(&keys.fvk);
        let nf_bytes = nullifier.to_bytes();
        assert!(
            nf_bytes.iter().any(|&b| b != 0),
            "nullifier must not be all zeros"
        );
    }

    #[test]
    fn test_empty_actions() {
        let keys = test_keys();
        let pivk = PreparedIncomingViewingKey::new(&keys.ivk);

        let found = scan_compact_actions(&pivk, &[]);
        assert!(found.is_empty(), "scanning empty actions must return empty");
    }
}

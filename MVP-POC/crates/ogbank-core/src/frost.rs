// RFC-OGB-001 §4.3: FROST Key Ceremony
//
// 2-of-3 Re-Randomized FROST over PallasBlake2b512 ciphersuite.
// Participants: Owner A (id=1), Owner B (id=2), Relayer (id=3).
//
// Two paths:
//   - DKG (recommended): no party ever sees the full `ask`
//   - Trusted Dealer (migration): splits an existing `ask` into shares

use reddsa::frost::redpallas::keys::{KeyPackage, PublicKeyPackage};
pub use reddsa::frost::redpallas::*;

use blake2b_simd::Params as Blake2bParams;

/// FROST configuration constants.
pub const MIN_SIGNERS: u16 = 2;
pub const MAX_SIGNERS: u16 = 3;

/// Participant identifiers (RFC-OGB-001 §3.1).
pub const OWNER_A_ID: u16 = 1;
pub const OWNER_B_ID: u16 = 2;
pub const RELAYER_ID: u16 = 3;

/// Result of a FROST key ceremony.
pub struct VaultKeyShares {
    /// Per-participant key packages (indexed by Identifier).
    pub key_packages: std::collections::BTreeMap<Identifier, KeyPackage>,
    /// Shared public key package (same for all participants).
    pub public_key_package: PublicKeyPackage,
}

/// Compute the vault ID per RFC-OGB-001 §4.3.3.
///
/// `vault_id = BLAKE2b-256("OGBank__VaultID_", ak || I2LEOSP_32(vault_nonce))`
pub fn compute_vault_id(ak_bytes: &[u8; 32], vault_nonce: u32) -> [u8; 32] {
    let mut result = [0u8; 32];
    let hash = Blake2bParams::new()
        .hash_length(32)
        .personal(b"OGBank__VaultID_")
        .to_state()
        .update(ak_bytes)
        .update(&vault_nonce.to_le_bytes())
        .finalize();
    result.copy_from_slice(hash.as_bytes());
    result
}

/// Generate vault key shares using the Trusted Dealer path.
///
/// RFC-OGB-001 §4.3.2: PERMITTED for migration from existing wallets.
/// The dealer momentarily holds the full secret — DKG is preferred for new vaults.
pub fn generate_with_dealer<R: rand::RngCore + rand::CryptoRng>(
    rng: &mut R,
) -> Result<VaultKeyShares, Box<dyn std::error::Error>> {
    let (shares, public_key_package) = keys::generate_with_dealer(
        MAX_SIGNERS,
        MIN_SIGNERS,
        keys::IdentifierList::Default,
        rng,
    )?;

    let mut key_packages = std::collections::BTreeMap::new();
    for (id, secret_share) in shares {
        let key_package = keys::KeyPackage::try_from(secret_share)?;
        key_packages.insert(id, key_package);
    }

    Ok(VaultKeyShares {
        key_packages,
        public_key_package,
    })
}

/// Run the full DKG ceremony in-process (for testing / single-machine setups).
///
/// RFC-OGB-001 §4.3.1: Each participant generates polynomials, exchanges
/// commitments and shares, then derives their KeyPackage.
///
/// In production, rounds 1-3 happen over authenticated encrypted channels.
/// This function simulates all three participants locally.
pub fn run_dkg_local<R: rand::RngCore + rand::CryptoRng>(
    rng: &mut R,
) -> Result<VaultKeyShares, Box<dyn std::error::Error>> {
    use frost_rerandomized::frost_core::frost::keys::dkg;
    use std::collections::{BTreeMap, HashMap};

    let participant_ids: Vec<Identifier> = (1..=MAX_SIGNERS)
        .map(|i| Identifier::try_from(i).unwrap())
        .collect();

    // --- Round 1: Each participant generates commitments ---
    let mut round1_secret_packages: HashMap<Identifier, dkg::round1::SecretPackage<PallasBlake2b512>> =
        HashMap::new();
    let mut round1_packages: HashMap<Identifier, dkg::round1::Package<PallasBlake2b512>> =
        HashMap::new();

    for &id in &participant_ids {
        let (secret_package, package) =
            dkg::part1::<PallasBlake2b512, _>(id, MAX_SIGNERS, MIN_SIGNERS, &mut *rng)?;
        round1_secret_packages.insert(id, secret_package);
        round1_packages.insert(id, package);
    }

    // --- Round 2: Each participant sends shares to others ---
    let mut round2_secret_packages: HashMap<Identifier, dkg::round2::SecretPackage<PallasBlake2b512>> =
        HashMap::new();
    // Map: recipient -> (sender -> round2 package)
    let mut all_round2_packages: HashMap<
        Identifier,
        HashMap<Identifier, dkg::round2::Package<PallasBlake2b512>>,
    > = HashMap::new();

    for &id in &participant_ids {
        all_round2_packages.insert(id, HashMap::new());
    }

    for &sender_id in &participant_ids {
        // Collect round1 packages from OTHER participants
        let received_round1: HashMap<Identifier, dkg::round1::Package<PallasBlake2b512>> =
            round1_packages
                .iter()
                .filter(|(&id, _)| id != sender_id)
                .map(|(&id, pkg)| (id, pkg.clone()))
                .collect();

        let secret = round1_secret_packages.remove(&sender_id).unwrap();
        let (secret_package, round2_pkgs) = dkg::part2(secret, &received_round1)?;
        round2_secret_packages.insert(sender_id, secret_package);

        // Distribute round2 packages to their recipients
        for (recipient_id, pkg) in round2_pkgs {
            all_round2_packages
                .get_mut(&recipient_id)
                .unwrap()
                .insert(sender_id, pkg);
        }
    }

    // --- Round 3: Each participant computes their key package ---
    let mut key_packages: BTreeMap<Identifier, KeyPackage> = BTreeMap::new();
    let mut public_key_package: Option<PublicKeyPackage> = None;

    for &id in &participant_ids {
        let received_round1: HashMap<Identifier, dkg::round1::Package<PallasBlake2b512>> =
            round1_packages
                .iter()
                .filter(|(&pid, _)| pid != id)
                .map(|(&pid, pkg)| (pid, pkg.clone()))
                .collect();

        let secret = round2_secret_packages.remove(&id).unwrap();
        let received_round2 = all_round2_packages.remove(&id).unwrap();

        let (key_package, pubkey_package) = dkg::part3(&secret, &received_round1, &received_round2)?;
        key_packages.insert(id, key_package);
        public_key_package = Some(pubkey_package);
    }

    Ok(VaultKeyShares {
        key_packages,
        public_key_package: public_key_package.unwrap(),
    })
}

/// Extract the group public key (`ak`) bytes from a PublicKeyPackage.
pub fn group_public_key_bytes(pubkey_package: &PublicKeyPackage) -> [u8; 32] {
    pubkey_package.group_public().serialize()
}

#[cfg(test)]
mod tests {
    use super::*;
    use frost_rerandomized::RandomizedParams;
    use std::collections::HashMap;

    #[test]
    fn test_dealer_produces_3_key_packages() {
        let mut rng = rand::rngs::OsRng;
        let vault = generate_with_dealer(&mut rng).unwrap();

        assert_eq!(vault.key_packages.len(), 3, "must produce 3 key packages");
    }

    #[test]
    fn test_dealer_all_share_same_verifying_key() {
        let mut rng = rand::rngs::OsRng;
        let vault = generate_with_dealer(&mut rng).unwrap();

        let group_vk = vault.public_key_package.group_public();
        for (_, kp) in &vault.key_packages {
            assert_eq!(
                kp.group_public(),
                group_vk,
                "all key packages must share the same group verifying key"
            );
        }
    }

    #[test]
    fn test_dealer_identifiers_are_1_2_3() {
        let mut rng = rand::rngs::OsRng;
        let vault = generate_with_dealer(&mut rng).unwrap();

        let ids: Vec<Identifier> = vault.key_packages.keys().cloned().collect();
        let expected: Vec<Identifier> = (1..=3)
            .map(|i| Identifier::try_from(i).unwrap())
            .collect();
        assert_eq!(ids, expected, "identifiers must be 1, 2, 3");
    }

    #[test]
    fn test_dkg_produces_3_key_packages() {
        let mut rng = rand::rngs::OsRng;
        let vault = run_dkg_local(&mut rng).unwrap();

        assert_eq!(vault.key_packages.len(), 3, "DKG must produce 3 key packages");
    }

    #[test]
    fn test_dkg_all_share_same_verifying_key() {
        let mut rng = rand::rngs::OsRng;
        let vault = run_dkg_local(&mut rng).unwrap();

        let group_vk = vault.public_key_package.group_public();
        for (_, kp) in &vault.key_packages {
            assert_eq!(
                kp.group_public(),
                group_vk,
                "DKG key packages must share the same group verifying key"
            );
        }
    }

    #[test]
    fn test_dkg_different_from_dealer() {
        let mut rng = rand::rngs::OsRng;
        let dealer = generate_with_dealer(&mut rng).unwrap();
        let dkg = run_dkg_local(&mut rng).unwrap();

        let dealer_ak = group_public_key_bytes(&dealer.public_key_package);
        let dkg_ak = group_public_key_bytes(&dkg.public_key_package);

        assert_ne!(
            dealer_ak, dkg_ak,
            "dealer and DKG should produce different group keys (different randomness)"
        );
    }

    #[test]
    fn test_vault_id_deterministic() {
        let ak = [42u8; 32];
        let id1 = compute_vault_id(&ak, 0);
        let id2 = compute_vault_id(&ak, 0);
        assert_eq!(id1, id2, "same inputs must produce same vault_id");
    }

    #[test]
    fn test_vault_id_different_nonces() {
        let ak = [42u8; 32];
        let id0 = compute_vault_id(&ak, 0);
        let id1 = compute_vault_id(&ak, 1);
        assert_ne!(id0, id1, "different nonces must produce different vault_ids");
    }

    #[test]
    fn test_2_of_3_signing_with_dealer() {
        use reddsa::frost::redpallas::{aggregate, round1, round2, SigningPackage};

        let mut rng = rand::rngs::OsRng;
        let vault = generate_with_dealer(&mut rng).unwrap();

        // Create randomized params for re-randomized FROST
        let randomized_params =
            RandomizedParams::new(&vault.public_key_package, &mut rng);
        let randomizer_point = randomized_params.randomizer_point();

        // Pick 2 participants: Owner A (1) and Relayer (3)
        let id1 = Identifier::try_from(OWNER_A_ID).unwrap();
        let id3 = Identifier::try_from(RELAYER_ID).unwrap();

        let kp1 = &vault.key_packages[&id1];
        let kp3 = &vault.key_packages[&id3];

        // Round 1: generate nonces
        let (nonces1, commitments1) = round1::commit(kp1.secret_share(), &mut rng);
        let (nonces3, commitments3) = round1::commit(kp3.secret_share(), &mut rng);

        let mut commitment_map = std::collections::BTreeMap::new();
        commitment_map.insert(id1, commitments1);
        commitment_map.insert(id3, commitments3);

        // Create signing package with a test message
        let message = b"test sighash for vault spend";
        let signing_package = SigningPackage::new(commitment_map, message);

        // Round 2: each signer produces a share (with randomizer point)
        let share1 = round2::sign(&signing_package, &nonces1, kp1, randomizer_point).unwrap();
        let share3 = round2::sign(&signing_package, &nonces3, kp3, randomizer_point).unwrap();

        let mut share_map = HashMap::new();
        share_map.insert(id1, share1);
        share_map.insert(id3, share3);

        // Aggregate with randomized params
        let signature =
            aggregate(&signing_package, &share_map, &vault.public_key_package, &randomized_params)
                .expect("aggregation must succeed");

        // Verify with the randomized group public key
        randomized_params
            .randomized_group_public_key()
            .verify(message, &signature)
            .expect("signature must be valid");
    }

    #[test]
    fn test_2_of_3_signing_with_dkg() {
        use reddsa::frost::redpallas::{aggregate, round1, round2, SigningPackage};

        let mut rng = rand::rngs::OsRng;
        let vault = run_dkg_local(&mut rng).unwrap();

        let randomized_params =
            RandomizedParams::new(&vault.public_key_package, &mut rng);
        let randomizer_point = randomized_params.randomizer_point();

        // Pick 2 participants: Owner B (2) and Relayer (3)
        let id2 = Identifier::try_from(OWNER_B_ID).unwrap();
        let id3 = Identifier::try_from(RELAYER_ID).unwrap();

        let kp2 = &vault.key_packages[&id2];
        let kp3 = &vault.key_packages[&id3];

        let (nonces2, commitments2) = round1::commit(kp2.secret_share(), &mut rng);
        let (nonces3, commitments3) = round1::commit(kp3.secret_share(), &mut rng);

        let mut commitment_map = std::collections::BTreeMap::new();
        commitment_map.insert(id2, commitments2);
        commitment_map.insert(id3, commitments3);

        let message = b"DKG signing test";
        let signing_package = SigningPackage::new(commitment_map, message);

        let share2 = round2::sign(&signing_package, &nonces2, kp2, randomizer_point).unwrap();
        let share3 = round2::sign(&signing_package, &nonces3, kp3, randomizer_point).unwrap();

        let mut share_map = HashMap::new();
        share_map.insert(id2, share2);
        share_map.insert(id3, share3);

        let signature =
            aggregate(&signing_package, &share_map, &vault.public_key_package, &randomized_params)
                .expect("DKG aggregation must succeed");

        randomized_params
            .randomized_group_public_key()
            .verify(message, &signature)
            .expect("DKG signature must be valid");
    }

    #[test]
    fn test_1_of_3_insufficient() {
        use reddsa::frost::redpallas::{aggregate, round1, round2, SigningPackage};

        let mut rng = rand::rngs::OsRng;
        let vault = generate_with_dealer(&mut rng).unwrap();

        let randomized_params =
            RandomizedParams::new(&vault.public_key_package, &mut rng);
        let randomizer_point = randomized_params.randomizer_point();

        // Only Relayer (1 share) — should fail at aggregation
        let id3 = Identifier::try_from(RELAYER_ID).unwrap();
        let kp3 = &vault.key_packages[&id3];

        let (nonces3, commitments3) = round1::commit(kp3.secret_share(), &mut rng);

        let mut commitment_map = std::collections::BTreeMap::new();
        commitment_map.insert(id3, commitments3);

        let message = b"relayer alone";
        let signing_package = SigningPackage::new(commitment_map, message);

        let share3 = round2::sign(&signing_package, &nonces3, kp3, randomizer_point).unwrap();

        let mut share_map = HashMap::new();
        share_map.insert(id3, share3);

        let result =
            aggregate(&signing_package, &share_map, &vault.public_key_package, &randomized_params);

        assert!(
            result.is_err(),
            "1-of-3 must not produce a valid signature"
        );
    }

    #[test]
    fn test_group_public_key_bytes_32() {
        let mut rng = rand::rngs::OsRng;
        let vault = generate_with_dealer(&mut rng).unwrap();

        let ak = group_public_key_bytes(&vault.public_key_package);
        assert_eq!(ak.len(), 32, "group public key must be 32 bytes");
        assert_ne!(ak, [0u8; 32], "group public key must not be zero");
    }
}

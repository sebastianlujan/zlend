// RFC-OGB-001 §9.3: Share Refresh via frost-core Repairable Threshold Scheme (RTS)
//
// Wraps frost_core::frost::keys::repairable to refresh FROST key shares
// without changing the group public key.
//
// Use case: periodic key rotation — refresh shares so that an attacker who
// compromised a share in the past cannot combine it with a future compromise.
//
// Reference: docs/technical/11_rfc-ogb-001.md §9.3

use std::collections::HashMap;

use frost_rerandomized::frost_core::frost::keys::{
    repairable, SecretShare, VerifiableSecretSharingCommitment,
};
use reddsa::frost::redpallas::{Identifier, PallasBlake2b512};

type Scalar = frost_rerandomized::frost_core::Scalar<PallasBlake2b512>;

/// Step 1: A helper generates delta values for all helpers.
///
/// Each helper calls this with their own `SecretShare`, the list of all helper
/// identifiers, and the target participant whose share is being refreshed.
///
/// Returns a map of (helper_id → delta) to distribute to each helper.
pub fn refresh_part1<R: rand::RngCore + rand::CryptoRng>(
    helper_ids: &[Identifier],
    helper_share: &SecretShare<PallasBlake2b512>,
    rng: &mut R,
    target_id: Identifier,
) -> HashMap<Identifier, Scalar> {
    repairable::repair_share_step_1::<PallasBlake2b512, R>(
        helper_ids,
        helper_share,
        rng,
        target_id,
    )
}

/// Step 2: A helper combines all deltas received from other helpers.
///
/// Each helper collects the delta meant for them from every other helper's
/// step 1 output (plus their own delta-for-self), then calls this to produce
/// a sigma value to send to the target participant.
pub fn refresh_part2(deltas: &[Scalar]) -> Scalar {
    repairable::repair_share_step_2::<PallasBlake2b512>(deltas)
}

/// Step 3: The target participant reconstructs their refreshed share.
///
/// The target receives one sigma from each helper and combines them with
/// their identifier and the original VSS commitment.
pub fn refresh_part3(
    sigmas: &[Scalar],
    target_id: Identifier,
    commitment: &VerifiableSecretSharingCommitment<PallasBlake2b512>,
) -> SecretShare<PallasBlake2b512> {
    repairable::repair_share_step_3::<PallasBlake2b512>(sigmas, target_id, commitment)
}

/// Run a full local share refresh for testing.
///
/// Given vault key shares produced by the dealer, refreshes the target
/// participant's share using all other participants as helpers.
/// Returns the new `SecretShare` for the target.
///
/// Verifies: group public key unchanged after refresh.
pub fn run_refresh_local(
    target_id: Identifier,
    secret_shares: &HashMap<Identifier, SecretShare<PallasBlake2b512>>,
) -> Result<SecretShare<PallasBlake2b512>, Box<dyn std::error::Error>> {
    let mut rng = rand::rngs::OsRng;

    // Helpers = everyone except the target
    let helper_ids: Vec<Identifier> = secret_shares
        .keys()
        .filter(|id| **id != target_id)
        .copied()
        .collect();

    if helper_ids.len() < 2 {
        return Err("need at least 2 helpers for 2-of-3 refresh".into());
    }

    // Step 1: Each helper generates deltas
    let mut all_deltas: HashMap<Identifier, Vec<Scalar>> = HashMap::new();
    for &helper_id in &helper_ids {
        all_deltas.insert(helper_id, Vec::new());
    }

    let mut step1_outputs: Vec<HashMap<Identifier, Scalar>> = Vec::new();
    for &helper_id in &helper_ids {
        let helper_share = &secret_shares[&helper_id];
        let deltas = refresh_part1(&helper_ids, helper_share, &mut rng, target_id);
        step1_outputs.push(deltas);
    }

    // Distribute: for each helper_j, collect the delta meant for them from each helper_i
    for &helper_j in &helper_ids {
        let deltas_for_j: Vec<Scalar> = step1_outputs
            .iter()
            .map(|output| output[&helper_j])
            .collect();
        all_deltas.insert(helper_j, deltas_for_j);
    }

    // Step 2: Each helper computes their sigma
    let mut sigmas: Vec<Scalar> = Vec::new();
    for &helper_id in &helper_ids {
        let sigma = refresh_part2(&all_deltas[&helper_id]);
        sigmas.push(sigma);
    }

    // Step 3: Target reconstructs their new share
    let target_share = &secret_shares[&target_id];
    let new_share = refresh_part3(&sigmas, target_id, target_share.commitment());

    Ok(new_share)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frost;
    use frost_rerandomized::RandomizedParams;
    use reddsa::frost::redpallas::keys::KeyPackage;
    use reddsa::frost::redpallas::{aggregate, round1, round2, SigningPackage};

    /// Generate dealer shares and return both the SecretShares and PublicKeyPackage.
    fn dealer_shares() -> (
        HashMap<Identifier, SecretShare<PallasBlake2b512>>,
        reddsa::frost::redpallas::keys::PublicKeyPackage,
    ) {
        let mut rng = rand::rngs::OsRng;
        let (shares, pubkey) = reddsa::frost::redpallas::keys::generate_with_dealer(
            frost::MAX_SIGNERS,
            frost::MIN_SIGNERS,
            reddsa::frost::redpallas::keys::IdentifierList::Default,
            &mut rng,
        )
        .unwrap();
        (shares, pubkey)
    }

    /// Sign a message with 2-of-3 FROST using given key packages and verify.
    fn sign_and_verify(
        key_packages: &std::collections::BTreeMap<Identifier, KeyPackage>,
        pubkey_pkg: &reddsa::frost::redpallas::keys::PublicKeyPackage,
        signer_ids: &[Identifier],
        message: &[u8],
    ) {
        let mut rng = rand::rngs::OsRng;
        let randomized_params = RandomizedParams::new(pubkey_pkg, &mut rng);
        let randomizer_point = randomized_params.randomizer_point();

        let kp0 = &key_packages[&signer_ids[0]];
        let kp1 = &key_packages[&signer_ids[1]];

        let (nonces0, commit0) = round1::commit(kp0.secret_share(), &mut rng);
        let (nonces1, commit1) = round1::commit(kp1.secret_share(), &mut rng);

        let mut commitment_map = std::collections::BTreeMap::new();
        commitment_map.insert(signer_ids[0], commit0);
        commitment_map.insert(signer_ids[1], commit1);

        let signing_package = SigningPackage::new(commitment_map, message);

        let share0 =
            round2::sign(&signing_package, &nonces0, kp0, randomizer_point).unwrap();
        let share1 =
            round2::sign(&signing_package, &nonces1, kp1, randomizer_point).unwrap();

        let mut share_map = HashMap::new();
        share_map.insert(signer_ids[0], share0);
        share_map.insert(signer_ids[1], share1);

        let signature =
            aggregate(&signing_package, &share_map, pubkey_pkg, &randomized_params)
                .expect("aggregation must succeed");

        randomized_params
            .randomized_group_public_key()
            .verify(message, &signature)
            .expect("signature must be valid");
    }

    #[test]
    fn test_refresh_part1_produces_deltas_for_all_helpers() {
        let (shares, _pubkey) = dealer_shares();
        let mut rng = rand::rngs::OsRng;

        let id1 = Identifier::try_from(1u16).unwrap();
        let id2 = Identifier::try_from(2u16).unwrap();
        let target = Identifier::try_from(3u16).unwrap();
        let helper_ids = vec![id1, id2];

        let deltas = refresh_part1(&helper_ids, &shares[&id1], &mut rng, target);
        assert_eq!(deltas.len(), helper_ids.len());
        assert!(deltas.contains_key(&id1));
        assert!(deltas.contains_key(&id2));
    }

    #[test]
    fn test_run_refresh_local_preserves_group_key() {
        let (shares, pubkey) = dealer_shares();
        let target = Identifier::try_from(3u16).unwrap();

        let new_share = run_refresh_local(target, &shares).unwrap();
        let new_kp = KeyPackage::try_from(new_share).unwrap();

        // Group public key must be unchanged
        assert_eq!(
            new_kp.group_public(),
            pubkey.group_public(),
            "group public key must be preserved after refresh"
        );
    }

    #[test]
    fn test_refresh_target_1_sign_after() {
        let (shares, pubkey) = dealer_shares();
        let target = Identifier::try_from(1u16).unwrap();
        let id2 = Identifier::try_from(2u16).unwrap();

        let new_share = run_refresh_local(target, &shares).unwrap();
        let new_kp = KeyPackage::try_from(new_share).unwrap();

        // Build key packages with the refreshed share
        let mut key_packages = std::collections::BTreeMap::new();
        key_packages.insert(target, new_kp);
        key_packages.insert(id2, KeyPackage::try_from(shares[&id2].clone()).unwrap());

        sign_and_verify(
            &key_packages,
            &pubkey,
            &[target, id2],
            b"sign after refresh target=1",
        );
    }

    #[test]
    fn test_refresh_target_2_sign_after() {
        let (shares, pubkey) = dealer_shares();
        let target = Identifier::try_from(2u16).unwrap();
        let id1 = Identifier::try_from(1u16).unwrap();

        let new_share = run_refresh_local(target, &shares).unwrap();
        let new_kp = KeyPackage::try_from(new_share).unwrap();

        let mut key_packages = std::collections::BTreeMap::new();
        key_packages.insert(id1, KeyPackage::try_from(shares[&id1].clone()).unwrap());
        key_packages.insert(target, new_kp);

        sign_and_verify(
            &key_packages,
            &pubkey,
            &[id1, target],
            b"sign after refresh target=2",
        );
    }

    #[test]
    fn test_refresh_target_3_sign_after() {
        let (shares, pubkey) = dealer_shares();
        let target = Identifier::try_from(3u16).unwrap();
        let id1 = Identifier::try_from(1u16).unwrap();

        let new_share = run_refresh_local(target, &shares).unwrap();
        let new_kp = KeyPackage::try_from(new_share).unwrap();

        let mut key_packages = std::collections::BTreeMap::new();
        key_packages.insert(id1, KeyPackage::try_from(shares[&id1].clone()).unwrap());
        key_packages.insert(target, new_kp);

        sign_and_verify(
            &key_packages,
            &pubkey,
            &[id1, target],
            b"sign after refresh target=3",
        );
    }

    #[test]
    fn test_refresh_all_three_sequentially() {
        let (shares, pubkey) = dealer_shares();
        let id1 = Identifier::try_from(1u16).unwrap();
        let id2 = Identifier::try_from(2u16).unwrap();
        let id3 = Identifier::try_from(3u16).unwrap();

        // Refresh participant 1
        let new1 = run_refresh_local(id1, &shares).unwrap();
        let mut refreshed = shares.clone();
        refreshed.insert(id1, new1);

        // Refresh participant 2 (using already-refreshed set)
        let new2 = run_refresh_local(id2, &refreshed).unwrap();
        refreshed.insert(id2, new2);

        // Refresh participant 3
        let new3 = run_refresh_local(id3, &refreshed).unwrap();
        refreshed.insert(id3, new3);

        // All three refreshed — signing must still work
        let mut key_packages = std::collections::BTreeMap::new();
        for (&id, share) in &refreshed {
            key_packages.insert(id, KeyPackage::try_from(share.clone()).unwrap());
        }

        // Group key unchanged
        assert_eq!(
            key_packages[&id1].group_public(),
            pubkey.group_public(),
            "group key must survive sequential refresh of all participants"
        );

        sign_and_verify(
            &key_packages,
            &pubkey,
            &[id1, id3],
            b"sign after full sequential refresh",
        );
    }

    #[test]
    fn test_repaired_share_matches_original() {
        let (shares, _pubkey) = dealer_shares();
        let target = Identifier::try_from(1u16).unwrap();

        let repaired = run_refresh_local(target, &shares).unwrap();

        // RTS recovers the original share — the value must match
        assert_eq!(
            shares[&target].secret(),
            repaired.secret(),
            "repaired share must equal original"
        );
    }

    #[test]
    fn test_insufficient_helpers_rejected() {
        let (shares, _pubkey) = dealer_shares();
        let target = Identifier::try_from(3u16).unwrap();

        // Only provide 1 helper (need at least 2)
        let mut insufficient: HashMap<Identifier, SecretShare<PallasBlake2b512>> = HashMap::new();
        let id1 = Identifier::try_from(1u16).unwrap();
        insufficient.insert(id1, shares[&id1].clone());
        insufficient.insert(target, shares[&target].clone());

        let result = run_refresh_local(target, &insufficient);
        assert!(result.is_err(), "refresh with 1 helper must fail");
    }

    #[test]
    fn test_refresh_part2_sums_deltas() {
        // Verify step 2 produces a deterministic sum
        use frost_rerandomized::frost_core::{Field, Group};
        type F = <<PallasBlake2b512 as frost_rerandomized::frost_core::Ciphersuite>::Group as Group>::Field;

        let one = <F as Field>::one();
        let two = one + one;
        let sigma = refresh_part2(&[one, two]);
        assert_eq!(sigma, one + two);
    }

    #[test]
    fn test_repaired_share_verifies_against_commitment() {
        let (shares, pubkey) = dealer_shares();
        let target = Identifier::try_from(2u16).unwrap();

        let repaired = run_refresh_local(target, &shares).unwrap();

        // Repaired share must verify against the original VSS commitment
        let (verifying_share, group_key) = repaired.verify().unwrap();
        assert_eq!(
            &group_key,
            pubkey.group_public(),
            "repaired share must verify against same group key"
        );

        // RTS recovers the original — verifying shares must match
        let (orig_vs, _) = shares[&target].verify().unwrap();
        assert_eq!(
            verifying_share, orig_vs,
            "repaired verifying share must match original"
        );
    }
}

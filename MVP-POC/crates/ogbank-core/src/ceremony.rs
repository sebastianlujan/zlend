// RFC-OGB-001 §7: Signing Ceremony Integration
//
// Connects FROST re-randomized signing (§4.3) with Signer validation (§7.3).
// Full flow:
//   1. Relayer constructs SigningRequest with authorization proof
//   2. Signer validates all 7 checks
//   3. On success, Signer produces FROST signature share
//   4. Relayer collects 2-of-3 shares and aggregates final signature
//
// This module orchestrates the end-to-end ceremony for a single vault spend.

use crate::frost::{self, Identifier, VaultKeyShares};
use crate::signer::{SignerState, SigningRequest, ValidationError};

use frost_rerandomized::RandomizedParams;
use reddsa::frost::redpallas::keys::{KeyPackage, PublicKeyPackage};
use reddsa::frost::redpallas::{self as redpallas, round1, round2, SigningPackage};
use std::collections::{BTreeMap, HashMap};

// ---------------------------------------------------------------------------
// Ceremony types
// ---------------------------------------------------------------------------

/// Pre-committed nonces for a signing ceremony participant.
#[derive(Clone)]
pub struct PreCommittedNonces {
    /// The signing nonces (kept secret by the signer).
    pub nonces: round1::SigningNonces,
    /// The signing commitments (shared with the coordinator).
    pub commitments: round1::SigningCommitments,
}

/// Result of a successful signing ceremony.
#[derive(Debug)]
pub struct CeremonyResult {
    /// The final aggregated signature.
    pub signature: redpallas::Signature,
    /// The randomized verifying key used for verification.
    pub randomized_verifying_key: redpallas::VerifyingKey,
}

/// Errors during the signing ceremony.
#[derive(Debug)]
pub enum CeremonyError {
    /// Signer validation failed.
    Validation(ValidationError),
    /// FROST signing error.
    Frost(String),
    /// Not enough signers (need at least 2-of-3).
    InsufficientSigners { have: usize, need: usize },
}

impl std::fmt::Display for CeremonyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Validation(e) => write!(f, "validation: {}", e),
            Self::Frost(e) => write!(f, "FROST: {}", e),
            Self::InsufficientSigners { have, need } => {
                write!(f, "need {} signers, have {}", need, have)
            }
        }
    }
}

impl std::error::Error for CeremonyError {}

impl From<ValidationError> for CeremonyError {
    fn from(e: ValidationError) -> Self {
        Self::Validation(e)
    }
}

// ---------------------------------------------------------------------------
// Ceremony coordinator (Relayer side)
// ---------------------------------------------------------------------------

/// Generate FROST nonce commitments for a participant (Round 1).
///
/// Called by each signer before the ceremony begins. The nonces are
/// stored locally; commitments are sent to the coordinator.
pub fn generate_nonces<R: rand::RngCore + rand::CryptoRng>(
    key_package: &KeyPackage,
    rng: &mut R,
) -> PreCommittedNonces {
    let (nonces, commitments) = round1::commit(key_package.secret_share(), rng);
    PreCommittedNonces {
        nonces,
        commitments,
    }
}

/// Produce a FROST signature share (Round 2) after validation.
///
/// This is the Signer's core operation:
///   1. Validate the signing request (7 checks)
///   2. Mark the nullifier as spent
///   3. Produce a FROST signature share with the randomizer point
pub fn validated_sign(
    signer_state: &mut SignerState,
    request: &SigningRequest,
    signing_package: &SigningPackage,
    nonces: &round1::SigningNonces,
    key_package: &KeyPackage,
    randomized_params: &RandomizedParams<reddsa::frost::redpallas::PallasBlake2b512>,
) -> Result<round2::SignatureShare, CeremonyError> {
    // Validate all 7 checks and mark nullifier spent
    signer_state.validate_and_accept(request)?;

    // Produce FROST signature share
    round2::sign(signing_package, nonces, key_package, randomized_params.randomizer_point())
        .map_err(|e| CeremonyError::Frost(format!("{}", e)))
}

/// Aggregate signature shares into a final signature (coordinator/Relayer).
///
/// Requires at least 2 signature shares (2-of-3 threshold).
pub fn aggregate_signature(
    signing_package: &SigningPackage,
    signature_shares: &HashMap<Identifier, round2::SignatureShare>,
    public_key_package: &PublicKeyPackage,
    randomized_params: &RandomizedParams<reddsa::frost::redpallas::PallasBlake2b512>,
) -> Result<CeremonyResult, CeremonyError> {
    if signature_shares.len() < frost::MIN_SIGNERS as usize {
        return Err(CeremonyError::InsufficientSigners {
            have: signature_shares.len(),
            need: frost::MIN_SIGNERS as usize,
        });
    }

    let signature =
        redpallas::aggregate(signing_package, signature_shares, public_key_package, randomized_params)
            .map_err(|e| CeremonyError::Frost(format!("{}", e)))?;

    let randomized_verifying_key = randomized_params.randomized_group_public_key().clone();

    Ok(CeremonyResult {
        signature,
        randomized_verifying_key,
    })
}

// ---------------------------------------------------------------------------
// End-to-end ceremony (local simulation for testing)
// ---------------------------------------------------------------------------

/// Run a complete signing ceremony locally with 2-of-3 participants.
///
/// This simulates the full protocol flow:
///   1. Relayer builds the signing request with auth proof
///   2. Two signers validate + produce shares
///   3. Relayer aggregates into final signature
///   4. Signature is verified
///
/// `signer_ids` must contain exactly 2 participant identifiers.
pub fn run_ceremony_local<R: rand::RngCore + rand::CryptoRng>(
    vault: &VaultKeyShares,
    signer_ids: &[Identifier; 2],
    request: &SigningRequest,
    signer_state: &mut SignerState,
    rng: &mut R,
) -> Result<CeremonyResult, CeremonyError> {
    // Create randomized params
    let randomized_params = RandomizedParams::new(&vault.public_key_package, &mut *rng);

    // Round 1: generate nonces for each signer
    let mut nonces_map: HashMap<Identifier, round1::SigningNonces> = HashMap::new();
    let mut commitment_map: BTreeMap<Identifier, round1::SigningCommitments> = BTreeMap::new();

    for &id in signer_ids {
        let kp = vault
            .key_packages
            .get(&id)
            .ok_or_else(|| CeremonyError::Frost(format!("no key package for {:?}", id)))?;
        let pre = generate_nonces(kp, &mut *rng);
        nonces_map.insert(id, pre.nonces);
        commitment_map.insert(id, pre.commitments);
    }

    // Build signing package from the SIGHASH (the message to sign)
    let signing_package = SigningPackage::new(commitment_map, &request.sighash);

    // Round 2: each signer validates + signs
    let mut share_map: HashMap<Identifier, round2::SignatureShare> = HashMap::new();

    for (i, &id) in signer_ids.iter().enumerate() {
        let kp = &vault.key_packages[&id];
        let nonces = nonces_map.remove(&id).unwrap();

        if i == 0 {
            // First signer: full validation + signing
            let share = validated_sign(
                signer_state,
                request,
                &signing_package,
                &nonces,
                kp,
                &randomized_params,
            )?;
            share_map.insert(id, share);
        } else {
            // Second signer: just signs (in production, they'd also validate independently)
            let share = round2::sign(&signing_package, &nonces, kp, randomized_params.randomizer_point())
                .map_err(|e| CeremonyError::Frost(format!("{}", e)))?;
            share_map.insert(id, share);
        }
    }

    // Aggregate
    aggregate_signature(
        &signing_package,
        &share_map,
        &vault.public_key_package,
        &randomized_params,
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{AuthTree, AuthorizationTicket};
    use crate::signer::{compute_sighash, AuthorizationProof, ProposedTx};

    /// Build a full valid ceremony setup.
    fn ceremony_setup() -> (
        VaultKeyShares,
        SignerState,
        SigningRequest,
        AuthorizationTicket,
    ) {
        let mut rng = rand::rngs::OsRng;
        let vault = frost::generate_with_dealer(&mut rng).unwrap();

        // Create auth ticket
        let auth_secret = [42u8; 32];
        let recipient = b"zcash_recipient_addr";
        let ticket = AuthorizationTicket::new(auth_secret, 1_000_000, recipient, 200_000);

        // Insert into auth tree
        let mut tree = AuthTree::new();
        let commitment = ticket.commitment();
        let idx = tree.insert(commitment).unwrap();
        let root = tree.root();
        let proof = tree.proof(idx).unwrap();

        // Build signing request
        let tx_data = b"ceremony_test_tx_data".to_vec();
        let sighash = compute_sighash(&tx_data);

        let request = SigningRequest {
            sighash,
            auth_proof: AuthorizationProof {
                auth_secret,
                auth_nullifier_hash: ticket.nullifier_hash(),
                max_amount: 1_000_000,
                destination_hash: ticket.destination_hash,
                expiry_block: 200_000,
                merkle_proof: proof,
            },
            proposed_tx: ProposedTx {
                total_spend_value: 500_000,
                recipient_address: recipient.to_vec(),
                tx_data,
            },
        };

        let signer_state = SignerState::new(root, 100_000);

        (vault, signer_state, request, ticket)
    }

    #[test]
    fn test_full_ceremony_owner_a_and_relayer() {
        let (vault, mut signer_state, request, _) = ceremony_setup();
        let mut rng = rand::rngs::OsRng;

        let signer_ids = [
            Identifier::try_from(frost::OWNER_A_ID).unwrap(),
            Identifier::try_from(frost::RELAYER_ID).unwrap(),
        ];

        let result = run_ceremony_local(&vault, &signer_ids, &request, &mut signer_state, &mut rng);
        assert!(result.is_ok(), "ceremony must succeed");

        let ceremony = result.unwrap();
        // Verify the signature
        ceremony
            .randomized_verifying_key
            .verify(&request.sighash, &ceremony.signature)
            .expect("signature must verify");
    }

    #[test]
    fn test_full_ceremony_owner_b_and_relayer() {
        let (vault, mut signer_state, request, _) = ceremony_setup();
        let mut rng = rand::rngs::OsRng;

        let signer_ids = [
            Identifier::try_from(frost::OWNER_B_ID).unwrap(),
            Identifier::try_from(frost::RELAYER_ID).unwrap(),
        ];

        let result = run_ceremony_local(&vault, &signer_ids, &request, &mut signer_state, &mut rng);
        assert!(result.is_ok());

        let ceremony = result.unwrap();
        ceremony
            .randomized_verifying_key
            .verify(&request.sighash, &ceremony.signature)
            .expect("signature must verify");
    }

    #[test]
    fn test_full_ceremony_both_owners() {
        let (vault, mut signer_state, request, _) = ceremony_setup();
        let mut rng = rand::rngs::OsRng;

        let signer_ids = [
            Identifier::try_from(frost::OWNER_A_ID).unwrap(),
            Identifier::try_from(frost::OWNER_B_ID).unwrap(),
        ];

        let result = run_ceremony_local(&vault, &signer_ids, &request, &mut signer_state, &mut rng);
        assert!(result.is_ok());

        let ceremony = result.unwrap();
        ceremony
            .randomized_verifying_key
            .verify(&request.sighash, &ceremony.signature)
            .expect("signature must verify");
    }

    #[test]
    fn test_ceremony_replay_rejected() {
        let (vault, mut signer_state, request, _) = ceremony_setup();
        let mut rng = rand::rngs::OsRng;

        let signer_ids = [
            Identifier::try_from(frost::OWNER_A_ID).unwrap(),
            Identifier::try_from(frost::RELAYER_ID).unwrap(),
        ];

        // First ceremony succeeds
        let result = run_ceremony_local(&vault, &signer_ids, &request, &mut signer_state, &mut rng);
        assert!(result.is_ok());

        // Replay: same request, same nullifier — must fail
        let result2 = run_ceremony_local(&vault, &signer_ids, &request, &mut signer_state, &mut rng);
        assert!(result2.is_err());
        match result2.unwrap_err() {
            CeremonyError::Validation(ValidationError::NullifierAlreadySpent) => {}
            e => panic!("expected NullifierAlreadySpent, got: {}", e),
        }
    }

    #[test]
    fn test_ceremony_wrong_secret_rejected() {
        let (vault, mut signer_state, mut request, _) = ceremony_setup();
        let mut rng = rand::rngs::OsRng;

        request.auth_proof.auth_secret = [99u8; 32]; // wrong secret

        let signer_ids = [
            Identifier::try_from(frost::OWNER_A_ID).unwrap(),
            Identifier::try_from(frost::RELAYER_ID).unwrap(),
        ];

        let result = run_ceremony_local(&vault, &signer_ids, &request, &mut signer_state, &mut rng);
        assert!(result.is_err());
    }

    #[test]
    fn test_ceremony_expired_ticket_rejected() {
        let (vault, mut signer_state, request, _) = ceremony_setup();
        let mut rng = rand::rngs::OsRng;

        signer_state.current_block_height = 300_000; // past expiry

        let signer_ids = [
            Identifier::try_from(frost::OWNER_A_ID).unwrap(),
            Identifier::try_from(frost::RELAYER_ID).unwrap(),
        ];

        let result = run_ceremony_local(&vault, &signer_ids, &request, &mut signer_state, &mut rng);
        assert!(result.is_err());
        match result.unwrap_err() {
            CeremonyError::Validation(ValidationError::TicketExpired { .. }) => {}
            e => panic!("expected TicketExpired, got: {}", e),
        }
    }

    #[test]
    fn test_ceremony_overspend_rejected() {
        let (vault, mut signer_state, mut request, _) = ceremony_setup();
        let mut rng = rand::rngs::OsRng;

        request.proposed_tx.total_spend_value = 2_000_000; // exceeds 1_000_000

        let signer_ids = [
            Identifier::try_from(frost::OWNER_A_ID).unwrap(),
            Identifier::try_from(frost::RELAYER_ID).unwrap(),
        ];

        let result = run_ceremony_local(&vault, &signer_ids, &request, &mut signer_state, &mut rng);
        assert!(result.is_err());
        match result.unwrap_err() {
            CeremonyError::Validation(ValidationError::AmountExceeded { .. }) => {}
            e => panic!("expected AmountExceeded, got: {}", e),
        }
    }

    #[test]
    fn test_aggregate_insufficient_shares() {
        let mut rng = rand::rngs::OsRng;
        let vault = frost::generate_with_dealer(&mut rng).unwrap();
        let randomized_params = RandomizedParams::new(&vault.public_key_package, &mut rng);

        // Empty commitment map (just need a valid signing package for the error test)
        let commitment_map = BTreeMap::new();
        let signing_package = SigningPackage::new(commitment_map, b"test");

        let share_map: HashMap<Identifier, round2::SignatureShare> = HashMap::new();

        let result = aggregate_signature(
            &signing_package,
            &share_map,
            &vault.public_key_package,
            &randomized_params,
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            CeremonyError::InsufficientSigners { have: 0, need: 2 } => {}
            e => panic!("expected InsufficientSigners, got: {}", e),
        }
    }

    #[test]
    fn test_generate_nonces_produces_commitments() {
        let mut rng = rand::rngs::OsRng;
        let vault = frost::generate_with_dealer(&mut rng).unwrap();
        let id1 = Identifier::try_from(frost::OWNER_A_ID).unwrap();
        let kp = &vault.key_packages[&id1];

        let pre = generate_nonces(kp, &mut rng);
        // Just verify it produces something (commitments are opaque)
        let _ = pre.commitments;
        let _ = pre.nonces;
    }

    #[test]
    fn test_two_sequential_tickets_both_succeed() {
        let mut rng = rand::rngs::OsRng;
        let vault = frost::generate_with_dealer(&mut rng).unwrap();
        let recipient = b"zcash_recipient_addr";

        // Two different tickets
        let ticket1 = AuthorizationTicket::new([1u8; 32], 500_000, recipient, 200_000);
        let ticket2 = AuthorizationTicket::new([2u8; 32], 700_000, recipient, 200_000);

        let mut tree = AuthTree::new();
        let idx1 = tree.insert(ticket1.commitment()).unwrap();
        let idx2 = tree.insert(ticket2.commitment()).unwrap();
        let root = tree.root();

        let tx_data = b"sequential_ceremony_test".to_vec();
        let sighash = compute_sighash(&tx_data);

        let mut signer_state = SignerState::new(root, 100_000);
        let signer_ids = [
            Identifier::try_from(frost::OWNER_A_ID).unwrap(),
            Identifier::try_from(frost::RELAYER_ID).unwrap(),
        ];

        // First ceremony
        let req1 = SigningRequest {
            sighash,
            auth_proof: AuthorizationProof {
                auth_secret: [1u8; 32],
                auth_nullifier_hash: ticket1.nullifier_hash(),
                max_amount: 500_000,
                destination_hash: ticket1.destination_hash,
                expiry_block: 200_000,
                merkle_proof: tree.proof(idx1).unwrap(),
            },
            proposed_tx: ProposedTx {
                total_spend_value: 400_000,
                recipient_address: recipient.to_vec(),
                tx_data: tx_data.clone(),
            },
        };

        let r1 = run_ceremony_local(&vault, &signer_ids, &req1, &mut signer_state, &mut rng);
        assert!(r1.is_ok(), "first ceremony must succeed");
        let c1 = r1.unwrap();
        c1.randomized_verifying_key
            .verify(&sighash, &c1.signature)
            .expect("first sig must verify");

        // Second ceremony (different ticket)
        let req2 = SigningRequest {
            sighash,
            auth_proof: AuthorizationProof {
                auth_secret: [2u8; 32],
                auth_nullifier_hash: ticket2.nullifier_hash(),
                max_amount: 700_000,
                destination_hash: ticket2.destination_hash,
                expiry_block: 200_000,
                merkle_proof: tree.proof(idx2).unwrap(),
            },
            proposed_tx: ProposedTx {
                total_spend_value: 600_000,
                recipient_address: recipient.to_vec(),
                tx_data,
            },
        };

        let r2 = run_ceremony_local(&vault, &signer_ids, &req2, &mut signer_state, &mut rng);
        assert!(r2.is_ok(), "second ceremony must succeed");
        let c2 = r2.unwrap();
        c2.randomized_verifying_key
            .verify(&sighash, &c2.signature)
            .expect("second sig must verify");

        assert_eq!(signer_state.spent_nullifiers.len(), 2);
    }
}

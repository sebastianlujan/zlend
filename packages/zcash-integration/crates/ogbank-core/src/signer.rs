// RFC-OGB-001 §7.3: Automated Signer Daemon
//
// The Signer holds KeyPackage₁ + FVK and validates signing requests with
// 7 deterministic checks before producing a signature share.
//
// Checks:
//   1. Commitment integrity
//   2. Merkle proof validity
//   3. Replay prevention (spent nullifier)
//   4. Nullifier derivation
//   5. Expiry
//   6a. Amount bound  /  6b. Destination match
//   7. SIGHASH consistency

use crate::auth::{self, AuthMerkleProof, SpentNullifierSet};
use blake2b_simd::Params as Blake2bParams;

// ---------------------------------------------------------------------------
// Domain separators (reused from auth, plus SIGHASH)
// ---------------------------------------------------------------------------

const PERSONAL_SIGHASH: &[u8; 16] = b"OGBank_SigHash__";

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

/// Authorization proof submitted by the Relayer to the Signer (§7.3).
#[derive(Clone, Debug)]
pub struct AuthorizationProof {
    /// The auth_secret (shared with Relayer).
    pub auth_secret: [u8; 32],
    /// Pre-computed `H(H(auth_secret))`.
    pub auth_nullifier_hash: [u8; 32],
    /// Maximum zatoshi authorized.
    pub max_amount: u64,
    /// `H(recipient_address)`.
    pub destination_hash: [u8; 32],
    /// Block height after which ticket is invalid.
    pub expiry_block: u32,
    /// Merkle inclusion proof for the commitment leaf.
    pub merkle_proof: AuthMerkleProof,
}

/// Proposed transaction parameters included in a signing request.
#[derive(Clone, Debug)]
pub struct ProposedTx {
    /// Total value being spent in zatoshi.
    pub total_spend_value: u64,
    /// Raw recipient address bytes.
    pub recipient_address: Vec<u8>,
    /// Serialized transaction data for SIGHASH computation.
    pub tx_data: Vec<u8>,
}

/// A signing request from the Relayer to the Signer (§7.3).
#[derive(Clone, Debug)]
pub struct SigningRequest {
    /// The SIGHASH the Relayer claims should be signed.
    pub sighash: [u8; 32],
    /// Authorization proof opening the commitment.
    pub auth_proof: AuthorizationProof,
    /// Proposed transaction details.
    pub proposed_tx: ProposedTx,
}

/// Errors from the 7 validation checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// CHECK 1: Reconstructed commitment doesn't match proof leaf.
    CommitmentMismatch,
    /// CHECK 2: Merkle proof doesn't verify against stored root.
    MerkleProofInvalid,
    /// CHECK 3: Nullifier already spent (replay attempt).
    NullifierAlreadySpent,
    /// CHECK 4: Nullifier hash derivation mismatch.
    NullifierDerivationMismatch,
    /// CHECK 5: Ticket has expired.
    TicketExpired {
        current_block: u32,
        expiry_block: u32,
    },
    /// CHECK 6a: Transaction amount exceeds ticket's max_amount.
    AmountExceeded {
        proposed: u64,
        max_allowed: u64,
    },
    /// CHECK 6b: Transaction destination doesn't match ticket's destination_hash.
    DestinationMismatch,
    /// CHECK 7: SIGHASH doesn't match independent computation from proposed_tx.
    SighashMismatch,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CommitmentMismatch => write!(f, "CHECK 1: commitment integrity failed"),
            Self::MerkleProofInvalid => write!(f, "CHECK 2: merkle proof invalid"),
            Self::NullifierAlreadySpent => write!(f, "CHECK 3: nullifier already spent"),
            Self::NullifierDerivationMismatch => {
                write!(f, "CHECK 4: nullifier derivation mismatch")
            }
            Self::TicketExpired {
                current_block,
                expiry_block,
            } => write!(
                f,
                "CHECK 5: ticket expired (current={}, expiry={})",
                current_block, expiry_block
            ),
            Self::AmountExceeded {
                proposed,
                max_allowed,
            } => write!(
                f,
                "CHECK 6a: amount {} exceeds max {}",
                proposed, max_allowed
            ),
            Self::DestinationMismatch => write!(f, "CHECK 6b: destination mismatch"),
            Self::SighashMismatch => write!(f, "CHECK 7: sighash mismatch"),
        }
    }
}

impl std::error::Error for ValidationError {}

// ---------------------------------------------------------------------------
// SIGHASH computation
// ---------------------------------------------------------------------------

/// Compute the SIGHASH from proposed transaction data.
///
/// `sighash = BLAKE2b-256("OGBank_SigHash__", tx_data)`
pub fn compute_sighash(tx_data: &[u8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    let hash = Blake2bParams::new()
        .hash_length(32)
        .personal(PERSONAL_SIGHASH)
        .to_state()
        .update(tx_data)
        .finalize();
    out.copy_from_slice(hash.as_bytes());
    out
}

// ---------------------------------------------------------------------------
// Signer state
// ---------------------------------------------------------------------------

/// The Signer's validation state.
///
/// Holds the auth tree root and spent nullifier set needed to perform
/// the 7 validation checks.
pub struct SignerState {
    /// The committed auth tree root (set during ticket batch acceptance).
    pub auth_tree_root: [u8; 32],
    /// Set of spent nullifier hashes (CHECK 3).
    pub spent_nullifiers: SpentNullifierSet,
    /// Current block height known to the Signer (for expiry checks).
    pub current_block_height: u32,
}

impl SignerState {
    /// Create a new signer state with the given auth tree root.
    pub fn new(auth_tree_root: [u8; 32], current_block_height: u32) -> Self {
        Self {
            auth_tree_root,
            spent_nullifiers: SpentNullifierSet::new(),
            current_block_height,
        }
    }

    /// Validate a signing request against all 7 checks (§7.3).
    ///
    /// Returns `Ok(())` if all checks pass. On failure, returns the
    /// specific check that failed. Does NOT mark the nullifier as spent
    /// on failure.
    pub fn validate(&self, request: &SigningRequest) -> Result<(), ValidationError> {
        let proof = &request.auth_proof;

        // CHECK 1 — Commitment Integrity
        let auth_nullifier = auth::derive_auth_nullifier(&proof.auth_secret);
        let expected_commitment = auth::compute_commitment(
            &auth_nullifier,
            &proof.auth_secret,
            proof.max_amount,
            &proof.destination_hash,
            proof.expiry_block,
        );

        // The leaf in the Merkle proof must match the reconstructed commitment
        // (we verify the proof against this commitment in CHECK 2)

        // CHECK 2 — Merkle Proof Validity
        if !proof
            .merkle_proof
            .verify(&expected_commitment, &self.auth_tree_root)
        {
            // If the commitment doesn't match OR the proof is invalid,
            // this catches both CHECK 1 and CHECK 2.
            // Distinguish: first check if the proof would verify with ANY leaf
            // by checking if the reconstructed commitment is plausible.
            // For simplicity, we report the more specific error.
            return Err(ValidationError::MerkleProofInvalid);
        }

        // CHECK 3 — Replay Prevention
        if self.spent_nullifiers.is_spent(&proof.auth_nullifier_hash) {
            return Err(ValidationError::NullifierAlreadySpent);
        }

        // CHECK 4 — Nullifier Derivation
        let expected_nullifier_hash = auth::derive_nullifier_hash(&auth_nullifier);
        if expected_nullifier_hash != proof.auth_nullifier_hash {
            return Err(ValidationError::NullifierDerivationMismatch);
        }

        // CHECK 5 — Expiry
        if self.current_block_height > proof.expiry_block {
            return Err(ValidationError::TicketExpired {
                current_block: self.current_block_height,
                expiry_block: proof.expiry_block,
            });
        }

        // CHECK 6a — Amount Bound
        if request.proposed_tx.total_spend_value > proof.max_amount {
            return Err(ValidationError::AmountExceeded {
                proposed: request.proposed_tx.total_spend_value,
                max_allowed: proof.max_amount,
            });
        }

        // CHECK 6b — Destination Match
        let tx_destination_hash =
            auth::derive_destination_hash(&request.proposed_tx.recipient_address);
        if tx_destination_hash != proof.destination_hash {
            return Err(ValidationError::DestinationMismatch);
        }

        // CHECK 7 — SIGHASH Consistency
        let expected_sighash = compute_sighash(&request.proposed_tx.tx_data);
        if expected_sighash != request.sighash {
            return Err(ValidationError::SighashMismatch);
        }

        Ok(())
    }

    /// Validate and accept a signing request.
    ///
    /// On success, marks the nullifier as spent and returns `Ok(())`.
    /// On failure, does NOT mark nullifier as spent.
    pub fn validate_and_accept(
        &mut self,
        request: &SigningRequest,
    ) -> Result<(), ValidationError> {
        self.validate(request)?;

        // All checks passed — mark nullifier as spent
        let spent = self
            .spent_nullifiers
            .mark_spent(request.auth_proof.auth_nullifier_hash);
        debug_assert!(spent, "nullifier was already checked in validate()");

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{AuthTree, AuthorizationTicket};

    /// Helper: create a valid signing request + signer state.
    fn valid_setup() -> (SignerState, SigningRequest) {
        // 1. Create a ticket
        let auth_secret = [42u8; 32];
        let max_amount = 1_000_000u64;
        let recipient = b"zcash_recipient_addr";
        let expiry_block = 200_000u32;
        let ticket = AuthorizationTicket::new(auth_secret, max_amount, recipient, expiry_block);

        // 2. Insert into tree
        let mut tree = AuthTree::new();
        let commitment = ticket.commitment();
        let idx = tree.insert(commitment).unwrap();
        let root = tree.root();
        let proof = tree.proof(idx).unwrap();

        // 3. Build proposed tx
        let tx_data = b"mock_transaction_data_for_sighash".to_vec();
        let sighash = compute_sighash(&tx_data);

        let proposed_tx = ProposedTx {
            total_spend_value: 500_000, // under max_amount
            recipient_address: recipient.to_vec(),
            tx_data,
        };

        // 4. Build auth proof
        let auth_proof = AuthorizationProof {
            auth_secret,
            auth_nullifier_hash: ticket.nullifier_hash(),
            max_amount,
            destination_hash: ticket.destination_hash,
            expiry_block,
            merkle_proof: proof,
        };

        // 5. Build signing request
        let request = SigningRequest {
            sighash,
            auth_proof,
            proposed_tx,
        };

        // 6. Signer state
        let state = SignerState::new(root, 100_000); // current block < expiry

        (state, request)
    }

    // --- Happy path ---

    #[test]
    fn test_valid_request_passes_all_checks() {
        let (state, request) = valid_setup();
        assert!(state.validate(&request).is_ok());
    }

    #[test]
    fn test_validate_and_accept_marks_spent() {
        let (mut state, request) = valid_setup();
        assert!(state.validate_and_accept(&request).is_ok());
        assert!(state
            .spent_nullifiers
            .is_spent(&request.auth_proof.auth_nullifier_hash));
    }

    // --- CHECK 1/2: Commitment integrity + Merkle proof ---

    #[test]
    fn test_check1_wrong_secret_fails() {
        let (state, mut request) = valid_setup();
        request.auth_proof.auth_secret = [99u8; 32]; // wrong secret
        let err = state.validate(&request).unwrap_err();
        assert_eq!(err, ValidationError::MerkleProofInvalid);
    }

    #[test]
    fn test_check2_wrong_root_fails() {
        let (mut state, request) = valid_setup();
        state.auth_tree_root = [0u8; 32]; // wrong root
        let err = state.validate(&request).unwrap_err();
        assert_eq!(err, ValidationError::MerkleProofInvalid);
    }

    #[test]
    fn test_check2_tampered_proof_fails() {
        let (state, mut request) = valid_setup();
        // Tamper with a sibling hash in the proof
        if let Some(entry) = request.auth_proof.merkle_proof.path.first_mut() {
            entry.0 = [0xFFu8; 32];
        }
        let err = state.validate(&request).unwrap_err();
        assert_eq!(err, ValidationError::MerkleProofInvalid);
    }

    // --- CHECK 3: Replay prevention ---

    #[test]
    fn test_check3_replay_rejected() {
        let (mut state, request) = valid_setup();
        // First request succeeds
        assert!(state.validate_and_accept(&request).is_ok());
        // Replay fails
        let err = state.validate(&request).unwrap_err();
        assert_eq!(err, ValidationError::NullifierAlreadySpent);
    }

    // --- CHECK 4: Nullifier derivation ---

    #[test]
    fn test_check4_wrong_nullifier_hash_fails() {
        let (state, mut request) = valid_setup();
        request.auth_proof.auth_nullifier_hash = [0xBBu8; 32]; // wrong hash
        let err = state.validate(&request).unwrap_err();
        // This will fail at CHECK 4 (nullifier derivation mismatch)
        // but may also fail at CHECK 2 if the proof doesn't verify
        assert!(
            err == ValidationError::NullifierDerivationMismatch
                || err == ValidationError::MerkleProofInvalid
        );
    }

    // --- CHECK 5: Expiry ---

    #[test]
    fn test_check5_expired_ticket_fails() {
        let (mut state, request) = valid_setup();
        state.current_block_height = 300_000; // past expiry of 200_000
        let err = state.validate(&request).unwrap_err();
        assert_eq!(
            err,
            ValidationError::TicketExpired {
                current_block: 300_000,
                expiry_block: 200_000,
            }
        );
    }

    #[test]
    fn test_check5_exact_expiry_passes() {
        let (mut state, request) = valid_setup();
        state.current_block_height = 200_000; // exactly at expiry
        assert!(state.validate(&request).is_ok());
    }

    // --- CHECK 6a: Amount bound ---

    #[test]
    fn test_check6a_overspend_fails() {
        let (state, mut request) = valid_setup();
        request.proposed_tx.total_spend_value = 2_000_000; // exceeds max_amount of 1_000_000
        let err = state.validate(&request).unwrap_err();
        assert_eq!(
            err,
            ValidationError::AmountExceeded {
                proposed: 2_000_000,
                max_allowed: 1_000_000,
            }
        );
    }

    #[test]
    fn test_check6a_exact_amount_passes() {
        let (state, mut request) = valid_setup();
        request.proposed_tx.total_spend_value = 1_000_000; // exactly at max
        assert!(state.validate(&request).is_ok());
    }

    // --- CHECK 6b: Destination match ---

    #[test]
    fn test_check6b_wrong_destination_fails() {
        let (state, mut request) = valid_setup();
        request.proposed_tx.recipient_address = b"wrong_address".to_vec();
        let err = state.validate(&request).unwrap_err();
        assert_eq!(err, ValidationError::DestinationMismatch);
    }

    // --- CHECK 7: SIGHASH consistency ---

    #[test]
    fn test_check7_wrong_sighash_fails() {
        let (state, mut request) = valid_setup();
        request.sighash = [0xAA; 32]; // doesn't match tx_data hash
        let err = state.validate(&request).unwrap_err();
        assert_eq!(err, ValidationError::SighashMismatch);
    }

    #[test]
    fn test_check7_tampered_tx_data_fails() {
        let (state, mut request) = valid_setup();
        // Keep the original sighash but change tx_data
        request.proposed_tx.tx_data = b"tampered_data".to_vec();
        let err = state.validate(&request).unwrap_err();
        assert_eq!(err, ValidationError::SighashMismatch);
    }

    // --- Sighash computation ---

    #[test]
    fn test_sighash_deterministic() {
        let data = b"test transaction";
        assert_eq!(compute_sighash(data), compute_sighash(data));
    }

    #[test]
    fn test_sighash_different_data() {
        assert_ne!(compute_sighash(b"tx1"), compute_sighash(b"tx2"));
    }

    // --- Multi-ticket scenario ---

    #[test]
    fn test_two_different_tickets_both_valid() {
        let recipient = b"zcash_recipient_addr";

        let ticket1 = AuthorizationTicket::new([1u8; 32], 500_000, recipient, 200_000);
        let ticket2 = AuthorizationTicket::new([2u8; 32], 700_000, recipient, 200_000);

        let mut tree = AuthTree::new();
        let idx1 = tree.insert(ticket1.commitment()).unwrap();
        let idx2 = tree.insert(ticket2.commitment()).unwrap();
        let root = tree.root();

        let tx_data = b"shared_tx_data".to_vec();
        let sighash = compute_sighash(&tx_data);

        let mut state = SignerState::new(root, 100_000);

        // First ticket
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

        assert!(state.validate_and_accept(&req1).is_ok());

        // Second ticket (different auth_secret, different nullifier)
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

        assert!(state.validate_and_accept(&req2).is_ok());
        assert_eq!(state.spent_nullifiers.len(), 2);
    }
}

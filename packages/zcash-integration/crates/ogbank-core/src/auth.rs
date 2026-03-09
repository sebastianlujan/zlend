// RFC-OGB-001 §5: Authorization Tickets
//
// Tornado Cash-inspired commitment-nullifier scheme with Merkle tree (depth 20).
// Tickets pre-authorize vault spends with amount bounds, destination locks,
// and expiry constraints. Double-hash nullifiers prevent replay.

use blake2b_simd::Params as Blake2bParams;
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Auth Merkle tree depth (supports 2^20 = 1,048,576 tickets per vault).
pub const TREE_DEPTH: usize = 20;

/// Maximum number of leaves in the auth tree.
pub const MAX_LEAVES: usize = 1 << TREE_DEPTH;

// Domain separators (16 bytes each, per RFC-OGB-001 Appendix A).
const PERSONAL_AUTH_NULL: &[u8; 16] = b"OGBank_AuthNull_";
const PERSONAL_AUTH_DEST: &[u8; 16] = b"OGBank_AuthDest\0";
const PERSONAL_NULL_HASH: &[u8; 16] = b"OGBank_NullHash\0";
const PERSONAL_AUTH_COMM: &[u8; 16] = b"OGBank_AuthComm_";
const PERSONAL_TREE_NODE: &[u8; 16] = b"OGBank_TreeNode\0";
const PERSONAL_TREE_LEAF: &[u8; 16] = b"OGBank_TreeLeaf\0";

// ---------------------------------------------------------------------------
// Hash helpers
// ---------------------------------------------------------------------------

fn blake2b_256(personalization: &[u8; 16], data: &[u8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    let hash = Blake2bParams::new()
        .hash_length(32)
        .personal(personalization)
        .to_state()
        .update(data)
        .finalize();
    out.copy_from_slice(hash.as_bytes());
    out
}

fn tree_node_hash(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut input = [0u8; 64];
    input[..32].copy_from_slice(left);
    input[32..].copy_from_slice(right);
    blake2b_256(PERSONAL_TREE_NODE, &input)
}

/// Precomputed empty leaf: `H("OGBank_TreeLeaf", [0u8; 32])`.
fn empty_leaf() -> [u8; 32] {
    blake2b_256(PERSONAL_TREE_LEAF, &[0u8; 32])
}

// ---------------------------------------------------------------------------
// Authorization Ticket
// ---------------------------------------------------------------------------

/// An authorization ticket pre-authorizing a vault spend (§5.1.1).
#[derive(Clone, Debug)]
pub struct AuthorizationTicket {
    /// Random secret shared with the Relayer.
    pub auth_secret: [u8; 32],
    /// Derived nullifier: `H("OGBank_AuthNull_", auth_secret)`.
    pub auth_nullifier: [u8; 32],
    /// Maximum zatoshi for this ticket.
    pub max_amount: u64,
    /// `H("OGBank_AuthDest", recipient_address_bytes)`.
    pub destination_hash: [u8; 32],
    /// Block height after which ticket is invalid.
    pub expiry_block: u32,
}

impl AuthorizationTicket {
    /// Create a new authorization ticket (§5.1.2).
    pub fn new(
        auth_secret: [u8; 32],
        max_amount: u64,
        recipient_address: &[u8],
        expiry_block: u32,
    ) -> Self {
        let auth_nullifier = derive_auth_nullifier(&auth_secret);
        let destination_hash = derive_destination_hash(recipient_address);
        Self {
            auth_secret,
            auth_nullifier,
            max_amount,
            destination_hash,
            expiry_block,
        }
    }

    /// Compute the ticket commitment (§5.2).
    pub fn commitment(&self) -> [u8; 32] {
        compute_commitment(
            &self.auth_nullifier,
            &self.auth_secret,
            self.max_amount,
            &self.destination_hash,
            self.expiry_block,
        )
    }

    /// Compute the nullifier hash for double-hash replay prevention (§5.1.3).
    pub fn nullifier_hash(&self) -> [u8; 32] {
        derive_nullifier_hash(&self.auth_nullifier)
    }
}

// ---------------------------------------------------------------------------
// Derivation functions
// ---------------------------------------------------------------------------

/// `auth_nullifier = BLAKE2b-256("OGBank_AuthNull_", auth_secret)`
pub fn derive_auth_nullifier(auth_secret: &[u8; 32]) -> [u8; 32] {
    blake2b_256(PERSONAL_AUTH_NULL, auth_secret)
}

/// `destination_hash = BLAKE2b-256("OGBank_AuthDest", recipient_address_bytes)`
pub fn derive_destination_hash(recipient_address: &[u8]) -> [u8; 32] {
    blake2b_256(PERSONAL_AUTH_DEST, recipient_address)
}

/// Double-hash: `auth_nullifier_hash = BLAKE2b-256("OGBank_NullHash", auth_nullifier)`
pub fn derive_nullifier_hash(auth_nullifier: &[u8; 32]) -> [u8; 32] {
    blake2b_256(PERSONAL_NULL_HASH, auth_nullifier)
}

/// Compute commitment per §5.2.
pub fn compute_commitment(
    auth_nullifier: &[u8; 32],
    auth_secret: &[u8; 32],
    max_amount: u64,
    destination_hash: &[u8; 32],
    expiry_block: u32,
) -> [u8; 32] {
    let mut preimage = Vec::with_capacity(32 + 32 + 8 + 32 + 4);
    preimage.extend_from_slice(auth_nullifier);
    preimage.extend_from_slice(auth_secret);
    preimage.extend_from_slice(&max_amount.to_le_bytes());
    preimage.extend_from_slice(destination_hash);
    preimage.extend_from_slice(&expiry_block.to_le_bytes());
    blake2b_256(PERSONAL_AUTH_COMM, &preimage)
}

// ---------------------------------------------------------------------------
// Merkle proof
// ---------------------------------------------------------------------------

/// A Merkle inclusion proof for an authorization commitment (§5.4).
#[derive(Clone, Debug)]
pub struct AuthMerkleProof {
    /// Index of the leaf in the tree.
    pub leaf_index: u32,
    /// Path of sibling hashes from leaf to root.
    /// Each entry: `(sibling_hash, is_left)` where `is_left` indicates the
    /// sibling is on the left side.
    pub path: Vec<([u8; 32], bool)>,
}

impl AuthMerkleProof {
    /// Verify this proof against a known root and leaf commitment.
    pub fn verify(&self, leaf: &[u8; 32], expected_root: &[u8; 32]) -> bool {
        let mut current = *leaf;
        for (sibling, sibling_is_left) in &self.path {
            current = if *sibling_is_left {
                tree_node_hash(sibling, &current)
            } else {
                tree_node_hash(&current, sibling)
            };
        }
        current == *expected_root
    }
}

// ---------------------------------------------------------------------------
// Auth Merkle Tree (incremental, append-only)
// ---------------------------------------------------------------------------

/// Incremental append-only Merkle tree for authorization commitments (§5.3).
pub struct AuthTree {
    /// Number of inserted leaves.
    next_index: usize,
    /// Stored leaves (commitments).
    leaves: Vec<[u8; 32]>,
    /// Precomputed zero hashes for each level.
    zero_hashes: Vec<[u8; 32]>,
}

impl AuthTree {
    /// Create a new empty auth tree.
    pub fn new() -> Self {
        let mut zero_hashes = vec![[0u8; 32]; TREE_DEPTH + 1];
        zero_hashes[0] = empty_leaf();
        for level in 1..=TREE_DEPTH {
            zero_hashes[level] = tree_node_hash(&zero_hashes[level - 1], &zero_hashes[level - 1]);
        }
        Self {
            next_index: 0,
            leaves: Vec::new(),
            zero_hashes,
        }
    }

    /// Number of leaves inserted so far.
    pub fn len(&self) -> usize {
        self.next_index
    }

    /// Whether the tree is empty.
    pub fn is_empty(&self) -> bool {
        self.next_index == 0
    }

    /// Insert a commitment leaf. Returns the leaf index.
    pub fn insert(&mut self, commitment: [u8; 32]) -> Result<u32, &'static str> {
        if self.next_index >= MAX_LEAVES {
            return Err("auth tree is full");
        }
        let index = self.next_index as u32;
        self.leaves.push(commitment);
        self.next_index += 1;
        Ok(index)
    }

    /// Compute the current root of the tree.
    pub fn root(&self) -> [u8; 32] {
        self.compute_root_from_leaves()
    }

    /// Generate a Merkle proof for the leaf at `index`.
    pub fn proof(&self, index: u32) -> Result<AuthMerkleProof, &'static str> {
        let index = index as usize;
        if index >= self.next_index {
            return Err("leaf index out of bounds");
        }

        let mut path = Vec::with_capacity(TREE_DEPTH);
        let mut current_index = index;

        // Build the full layer of hashes at each level
        let mut current_layer: Vec<[u8; 32]> = self
            .leaves
            .iter()
            .copied()
            .chain(std::iter::repeat_n(self.zero_hashes[0], MAX_LEAVES - self.next_index))
            .take(MAX_LEAVES)
            .collect();

        for level in 0..TREE_DEPTH {
            let sibling_index = current_index ^ 1;
            let sibling_hash = if sibling_index < current_layer.len() {
                current_layer[sibling_index]
            } else {
                self.zero_hashes[level]
            };
            // sibling_is_left means the sibling is to the left of current node
            let sibling_is_left = sibling_index < current_index;
            path.push((sibling_hash, sibling_is_left));

            // Compute next layer
            let next_len = current_layer.len() / 2;
            let mut next_layer = Vec::with_capacity(next_len);
            for i in 0..next_len {
                next_layer.push(tree_node_hash(&current_layer[2 * i], &current_layer[2 * i + 1]));
            }
            current_layer = next_layer;
            current_index /= 2;
        }

        Ok(AuthMerkleProof {
            leaf_index: index as u32,
            path,
        })
    }

    fn compute_root_from_leaves(&self) -> [u8; 32] {
        if self.next_index == 0 {
            return self.zero_hashes[TREE_DEPTH];
        }

        let mut current_layer: Vec<[u8; 32]> = self
            .leaves
            .iter()
            .copied()
            .chain(std::iter::repeat_n(self.zero_hashes[0], MAX_LEAVES - self.next_index))
            .take(MAX_LEAVES)
            .collect();

        for _level in 0..TREE_DEPTH {
            let next_len = current_layer.len() / 2;
            let mut next_layer = Vec::with_capacity(next_len);
            for i in 0..next_len {
                next_layer.push(tree_node_hash(&current_layer[2 * i], &current_layer[2 * i + 1]));
            }
            current_layer = next_layer;
        }

        current_layer[0]
    }
}

impl Default for AuthTree {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Spent Nullifier Set (simplified for MVP; production uses SMT §5.5)
// ---------------------------------------------------------------------------

/// Tracks consumed authorization ticket nullifiers.
///
/// MVP uses a HashSet. Production replaces this with a Sparse Merkle Tree
/// (depth 256) per RFC-OGB-001 §5.5.
pub struct SpentNullifierSet {
    spent: HashSet<[u8; 32]>,
}

impl SpentNullifierSet {
    pub fn new() -> Self {
        Self {
            spent: HashSet::new(),
        }
    }

    /// Check if a nullifier hash has already been spent.
    pub fn is_spent(&self, nullifier_hash: &[u8; 32]) -> bool {
        self.spent.contains(nullifier_hash)
    }

    /// Mark a nullifier hash as spent. Returns `false` if already spent.
    pub fn mark_spent(&mut self, nullifier_hash: [u8; 32]) -> bool {
        self.spent.insert(nullifier_hash)
    }

    pub fn len(&self) -> usize {
        self.spent.len()
    }

    pub fn is_empty(&self) -> bool {
        self.spent.is_empty()
    }
}

impl Default for SpentNullifierSet {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_ticket() -> AuthorizationTicket {
        AuthorizationTicket::new(
            [42u8; 32],
            1_000_000,        // 0.01 ZEC
            b"zcash_address", // mock recipient
            100_000,          // expires at block 100k
        )
    }

    // --- Derivation tests ---

    #[test]
    fn test_auth_nullifier_deterministic() {
        let secret = [1u8; 32];
        let n1 = derive_auth_nullifier(&secret);
        let n2 = derive_auth_nullifier(&secret);
        assert_eq!(n1, n2);
        assert_ne!(n1, [0u8; 32]);
    }

    #[test]
    fn test_auth_nullifier_different_secrets() {
        let n1 = derive_auth_nullifier(&[1u8; 32]);
        let n2 = derive_auth_nullifier(&[2u8; 32]);
        assert_ne!(n1, n2);
    }

    #[test]
    fn test_nullifier_hash_double_hash() {
        let secret = [7u8; 32];
        let nullifier = derive_auth_nullifier(&secret);
        let null_hash = derive_nullifier_hash(&nullifier);

        // Verify it's different from the nullifier itself (double hash)
        assert_ne!(null_hash, nullifier);
        // Verify determinism
        assert_eq!(null_hash, derive_nullifier_hash(&nullifier));
    }

    #[test]
    fn test_destination_hash_deterministic() {
        let addr = b"t1exampleaddress";
        let h1 = derive_destination_hash(addr);
        let h2 = derive_destination_hash(addr);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_destination_hash_different_addresses() {
        let h1 = derive_destination_hash(b"addr_a");
        let h2 = derive_destination_hash(b"addr_b");
        assert_ne!(h1, h2);
    }

    // --- Commitment tests ---

    #[test]
    fn test_commitment_deterministic() {
        let ticket = test_ticket();
        let c1 = ticket.commitment();
        let c2 = ticket.commitment();
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_commitment_changes_with_amount() {
        let t1 = AuthorizationTicket::new([42u8; 32], 100, b"addr", 1000);
        let t2 = AuthorizationTicket::new([42u8; 32], 200, b"addr", 1000);
        assert_ne!(t1.commitment(), t2.commitment());
    }

    #[test]
    fn test_commitment_changes_with_expiry() {
        let t1 = AuthorizationTicket::new([42u8; 32], 100, b"addr", 1000);
        let t2 = AuthorizationTicket::new([42u8; 32], 100, b"addr", 2000);
        assert_ne!(t1.commitment(), t2.commitment());
    }

    #[test]
    fn test_commitment_changes_with_destination() {
        let t1 = AuthorizationTicket::new([42u8; 32], 100, b"addr_a", 1000);
        let t2 = AuthorizationTicket::new([42u8; 32], 100, b"addr_b", 1000);
        assert_ne!(t1.commitment(), t2.commitment());
    }

    #[test]
    fn test_commitment_changes_with_secret() {
        let t1 = AuthorizationTicket::new([1u8; 32], 100, b"addr", 1000);
        let t2 = AuthorizationTicket::new([2u8; 32], 100, b"addr", 1000);
        assert_ne!(t1.commitment(), t2.commitment());
    }

    // --- Tree tests ---

    #[test]
    fn test_empty_tree_has_deterministic_root() {
        let tree1 = AuthTree::new();
        let tree2 = AuthTree::new();
        assert_eq!(tree1.root(), tree2.root());
        assert_ne!(tree1.root(), [0u8; 32]);
    }

    #[test]
    fn test_insert_changes_root() {
        let mut tree = AuthTree::new();
        let empty_root = tree.root();
        let ticket = test_ticket();
        tree.insert(ticket.commitment()).unwrap();
        assert_ne!(tree.root(), empty_root);
    }

    #[test]
    fn test_insert_increments_len() {
        let mut tree = AuthTree::new();
        assert_eq!(tree.len(), 0);
        assert!(tree.is_empty());
        tree.insert([1u8; 32]).unwrap();
        assert_eq!(tree.len(), 1);
        assert!(!tree.is_empty());
    }

    #[test]
    fn test_proof_verifies() {
        let mut tree = AuthTree::new();
        let ticket = test_ticket();
        let commitment = ticket.commitment();
        let idx = tree.insert(commitment).unwrap();

        let root = tree.root();
        let proof = tree.proof(idx).unwrap();

        assert!(proof.verify(&commitment, &root));
    }

    #[test]
    fn test_proof_fails_wrong_leaf() {
        let mut tree = AuthTree::new();
        let commitment = test_ticket().commitment();
        let idx = tree.insert(commitment).unwrap();

        let root = tree.root();
        let proof = tree.proof(idx).unwrap();

        let wrong_leaf = [99u8; 32];
        assert!(!proof.verify(&wrong_leaf, &root));
    }

    #[test]
    fn test_proof_fails_wrong_root() {
        let mut tree = AuthTree::new();
        let commitment = test_ticket().commitment();
        let idx = tree.insert(commitment).unwrap();

        let proof = tree.proof(idx).unwrap();
        let wrong_root = [99u8; 32];
        assert!(!proof.verify(&commitment, &wrong_root));
    }

    #[test]
    fn test_multiple_insertions_all_provable() {
        let mut tree = AuthTree::new();
        let mut commitments = Vec::new();

        for i in 0u8..5 {
            let ticket = AuthorizationTicket::new([i; 32], 100, b"addr", 1000);
            let c = ticket.commitment();
            tree.insert(c).unwrap();
            commitments.push(c);
        }

        let root = tree.root();

        for (i, c) in commitments.iter().enumerate() {
            let proof = tree.proof(i as u32).unwrap();
            assert!(
                proof.verify(c, &root),
                "proof for leaf {} must verify",
                i
            );
        }
    }

    #[test]
    fn test_proof_path_length() {
        let mut tree = AuthTree::new();
        tree.insert([1u8; 32]).unwrap();
        let proof = tree.proof(0).unwrap();
        assert_eq!(proof.path.len(), TREE_DEPTH);
    }

    #[test]
    fn test_proof_out_of_bounds() {
        let tree = AuthTree::new();
        assert!(tree.proof(0).is_err());
    }

    // --- Spent nullifier set tests ---

    #[test]
    fn test_nullifier_set_initially_empty() {
        let set = SpentNullifierSet::new();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
        assert!(!set.is_spent(&[1u8; 32]));
    }

    #[test]
    fn test_nullifier_mark_spent() {
        let mut set = SpentNullifierSet::new();
        let nh = [1u8; 32];
        assert!(set.mark_spent(nh)); // first time -> true
        assert!(set.is_spent(&nh));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_nullifier_double_spend_rejected() {
        let mut set = SpentNullifierSet::new();
        let nh = [1u8; 32];
        assert!(set.mark_spent(nh));
        assert!(!set.mark_spent(nh)); // second time -> false
    }

    // --- Integration test ---

    #[test]
    fn test_full_ticket_lifecycle() {
        // 1. Create ticket
        let mut rng_secret = [0u8; 32];
        rng_secret[0] = 0xAB;
        let ticket = AuthorizationTicket::new(
            rng_secret,
            5_000_000, // 0.05 ZEC
            b"recipient_orchard_address",
            200_000,
        );

        // 2. Insert commitment into tree
        let mut tree = AuthTree::new();
        let commitment = ticket.commitment();
        let idx = tree.insert(commitment).unwrap();

        // 3. Generate proof
        let root = tree.root();
        let proof = tree.proof(idx).unwrap();
        assert!(proof.verify(&commitment, &root));

        // 4. Verify nullifier chain
        let null_hash = ticket.nullifier_hash();
        let expected_null = derive_auth_nullifier(&ticket.auth_secret);
        let expected_null_hash = derive_nullifier_hash(&expected_null);
        assert_eq!(null_hash, expected_null_hash);

        // 5. Spend nullifier (first time succeeds)
        let mut spent = SpentNullifierSet::new();
        assert!(!spent.is_spent(&null_hash));
        assert!(spent.mark_spent(null_hash));

        // 6. Replay rejected
        assert!(spent.is_spent(&null_hash));
        assert!(!spent.mark_spent(null_hash));
    }
}

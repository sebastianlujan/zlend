// RFC-OGB-001 §7.2.1, §11.1: Nonce Registry
//
// FROST nonce reuse enables complete key recovery. This module provides
// a registry that binds pre-committed nonces to specific authorization
// tickets via `nonce_registry[auth_nullifier_hash]`.
//
// Each nonce pair is SINGLE-USE. After producing a signature share,
// the entry is deleted before returning the share.
//
// Security invariants:
//   - Each auth_nullifier_hash maps to exactly one (SigningNonces, SigningCommitments)
//   - Once consumed, the entry is removed and cannot be reused
//   - If the signer crashes after signing but before deletion, it MUST
//     check on restart whether the nullifier was spent on-chain

use reddsa::frost::redpallas::{round1, keys::KeyPackage};
use std::collections::HashMap;

/// A nonce entry bound to a specific authorization ticket.
#[derive(Clone)]
pub struct NonceEntry {
    /// The signing nonces (secret — never leaves the signer).
    pub nonces: round1::SigningNonces,
    /// The signing commitments (shared with the coordinator/relayer).
    pub commitments: round1::SigningCommitments,
}

/// Registry mapping auth_nullifier_hash → pre-committed nonce pair.
///
/// Each ticket in a batch gets exactly one nonce pair. The relayer
/// receives only the commitments; the signer retains the nonces.
pub struct NonceRegistry {
    entries: HashMap<[u8; 32], NonceEntry>,
}

impl NonceRegistry {
    /// Create an empty nonce registry.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Number of pre-committed nonces available.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Pre-commit a nonce pair for a specific ticket.
    ///
    /// The `auth_nullifier_hash` binds this nonce to exactly one ticket.
    /// Returns `false` if a nonce is already registered for this hash
    /// (duplicate registration is a programming error).
    pub fn register(
        &mut self,
        auth_nullifier_hash: [u8; 32],
        entry: NonceEntry,
    ) -> bool {
        if self.entries.contains_key(&auth_nullifier_hash) {
            return false;
        }
        self.entries.insert(auth_nullifier_hash, entry);
        true
    }

    /// Pre-commit a nonce for a ticket by generating fresh nonces from the key package.
    ///
    /// Returns the commitments (to be shared with the relayer).
    pub fn register_fresh<R: rand::RngCore + rand::CryptoRng>(
        &mut self,
        auth_nullifier_hash: [u8; 32],
        key_package: &KeyPackage,
        rng: &mut R,
    ) -> Option<round1::SigningCommitments> {
        if self.entries.contains_key(&auth_nullifier_hash) {
            return None;
        }

        let (nonces, commitments) = round1::commit(key_package.secret_share(), rng);
        self.entries.insert(
            auth_nullifier_hash,
            NonceEntry {
                nonces,
                commitments: commitments.clone(),
            },
        );
        Some(commitments)
    }

    /// Look up the commitments for a ticket (without consuming).
    pub fn get_commitments(&self, auth_nullifier_hash: &[u8; 32]) -> Option<round1::SigningCommitments> {
        self.entries.get(auth_nullifier_hash).map(|e| e.commitments.clone())
    }

    /// Check whether a nonce is registered for a ticket.
    pub fn contains(&self, auth_nullifier_hash: &[u8; 32]) -> bool {
        self.entries.contains_key(auth_nullifier_hash)
    }

    /// Consume (take) the nonce for a ticket.
    ///
    /// This removes the entry from the registry. The nonce MUST NOT be
    /// used again. Returns `None` if no nonce was registered.
    ///
    /// Per §11.1: the entry is deleted BEFORE the signature share is
    /// returned to prevent reuse on crash recovery.
    pub fn consume(&mut self, auth_nullifier_hash: &[u8; 32]) -> Option<NonceEntry> {
        self.entries.remove(auth_nullifier_hash)
    }

    /// Batch pre-commit nonces for multiple tickets.
    ///
    /// Returns a vec of (auth_nullifier_hash, commitments) for each
    /// successfully registered ticket. Skips duplicates.
    pub fn register_batch<R: rand::RngCore + rand::CryptoRng>(
        &mut self,
        nullifier_hashes: &[[u8; 32]],
        key_package: &KeyPackage,
        rng: &mut R,
    ) -> Vec<([u8; 32], round1::SigningCommitments)> {
        let mut results = Vec::with_capacity(nullifier_hashes.len());
        for &hash in nullifier_hashes {
            if let Some(commitments) = self.register_fresh(hash, key_package, rng) {
                results.push((hash, commitments));
            }
        }
        results
    }

    /// Remove all entries (e.g., on vault revocation).
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

// ---------------------------------------------------------------------------
// Relayer Nonce Pool (§7.2.1)
// ---------------------------------------------------------------------------

/// A pool of pre-generated nonces for the relayer's own FROST signing.
///
/// Unlike the signer's nonce registry (which binds nonces to tickets),
/// the relayer pool is a simple FIFO queue of fresh nonce pairs.
pub struct NoncePool {
    pool: Vec<NonceEntry>,
}

impl NoncePool {
    /// Create an empty nonce pool.
    pub fn new() -> Self {
        Self { pool: Vec::new() }
    }

    /// Number of nonces available.
    pub fn len(&self) -> usize {
        self.pool.len()
    }

    /// Whether the pool is empty.
    pub fn is_empty(&self) -> bool {
        self.pool.is_empty()
    }

    /// Generate and add `count` nonces to the pool.
    pub fn refill<R: rand::RngCore + rand::CryptoRng>(
        &mut self,
        count: usize,
        key_package: &KeyPackage,
        rng: &mut R,
    ) {
        for _ in 0..count {
            let (nonces, commitments) = round1::commit(key_package.secret_share(), rng);
            self.pool.push(NonceEntry { nonces, commitments });
        }
    }

    /// Take the next available nonce pair. Returns `None` if pool is empty.
    pub fn take(&mut self) -> Option<NonceEntry> {
        self.pool.pop()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frost;

    fn setup() -> (frost::VaultKeyShares, KeyPackage) {
        let mut rng = rand::rngs::OsRng;
        let vault = frost::generate_with_dealer(&mut rng).unwrap();
        let id1 = frost::Identifier::try_from(frost::OWNER_A_ID).unwrap();
        let kp = vault.key_packages[&id1].clone();
        (vault, kp)
    }

    #[test]
    fn test_registry_register_and_consume() {
        let (_vault, kp) = setup();
        let mut rng = rand::rngs::OsRng;
        let mut registry = NonceRegistry::new();

        let hash = [1u8; 32];
        let commitments = registry.register_fresh(hash, &kp, &mut rng);
        assert!(commitments.is_some(), "first registration must succeed");
        assert_eq!(registry.len(), 1);

        // Consume
        let entry = registry.consume(&hash);
        assert!(entry.is_some(), "consume must return the entry");
        assert!(registry.is_empty(), "registry must be empty after consume");
    }

    #[test]
    fn test_registry_duplicate_rejected() {
        let (_vault, kp) = setup();
        let mut rng = rand::rngs::OsRng;
        let mut registry = NonceRegistry::new();

        let hash = [2u8; 32];
        assert!(registry.register_fresh(hash, &kp, &mut rng).is_some());
        assert!(
            registry.register_fresh(hash, &kp, &mut rng).is_none(),
            "duplicate registration must be rejected"
        );
        assert_eq!(registry.len(), 1, "must still have exactly 1 entry");
    }

    #[test]
    fn test_registry_consume_removes_entry() {
        let (_vault, kp) = setup();
        let mut rng = rand::rngs::OsRng;
        let mut registry = NonceRegistry::new();

        let hash = [3u8; 32];
        registry.register_fresh(hash, &kp, &mut rng);

        // Consume once
        assert!(registry.consume(&hash).is_some());
        // Second consume must return None
        assert!(
            registry.consume(&hash).is_none(),
            "second consume must return None (single-use)"
        );
    }

    #[test]
    fn test_registry_contains() {
        let (_vault, kp) = setup();
        let mut rng = rand::rngs::OsRng;
        let mut registry = NonceRegistry::new();

        let hash = [4u8; 32];
        assert!(!registry.contains(&hash));
        registry.register_fresh(hash, &kp, &mut rng);
        assert!(registry.contains(&hash));
    }

    #[test]
    fn test_registry_get_commitments() {
        let (_vault, kp) = setup();
        let mut rng = rand::rngs::OsRng;
        let mut registry = NonceRegistry::new();

        let hash = [5u8; 32];
        let registered = registry.register_fresh(hash, &kp, &mut rng).unwrap();

        let fetched = registry.get_commitments(&hash).unwrap();
        // Commitments should be the same reference (clone)
        assert_eq!(
            format!("{:?}", registered),
            format!("{:?}", fetched),
            "get_commitments must return the same commitments"
        );
    }

    #[test]
    fn test_registry_batch() {
        let (_vault, kp) = setup();
        let mut rng = rand::rngs::OsRng;
        let mut registry = NonceRegistry::new();

        let hashes: Vec<[u8; 32]> = (0..5).map(|i| [i as u8; 32]).collect();
        let results = registry.register_batch(&hashes, &kp, &mut rng);

        assert_eq!(results.len(), 5, "batch must register 5 nonces");
        assert_eq!(registry.len(), 5);
    }

    #[test]
    fn test_registry_batch_skips_duplicates() {
        let (_vault, kp) = setup();
        let mut rng = rand::rngs::OsRng;
        let mut registry = NonceRegistry::new();

        // Pre-register one
        let hash0 = [0u8; 32];
        registry.register_fresh(hash0, &kp, &mut rng);

        // Batch includes the duplicate
        let hashes: Vec<[u8; 32]> = (0..3).map(|i| [i as u8; 32]).collect();
        let results = registry.register_batch(&hashes, &kp, &mut rng);

        assert_eq!(results.len(), 2, "batch must skip 1 duplicate");
        assert_eq!(registry.len(), 3, "total must be 3");
    }

    #[test]
    fn test_registry_clear() {
        let (_vault, kp) = setup();
        let mut rng = rand::rngs::OsRng;
        let mut registry = NonceRegistry::new();

        let hashes: Vec<[u8; 32]> = (0..3).map(|i| [i as u8; 32]).collect();
        registry.register_batch(&hashes, &kp, &mut rng);
        assert_eq!(registry.len(), 3);

        registry.clear();
        assert!(registry.is_empty(), "clear must empty the registry");
    }

    // --- NoncePool tests ---

    #[test]
    fn test_pool_refill_and_take() {
        let (_vault, kp) = setup();
        let mut rng = rand::rngs::OsRng;
        let mut pool = NoncePool::new();

        assert!(pool.is_empty());
        pool.refill(5, &kp, &mut rng);
        assert_eq!(pool.len(), 5);

        let entry = pool.take();
        assert!(entry.is_some());
        assert_eq!(pool.len(), 4);
    }

    #[test]
    fn test_pool_exhaustion() {
        let (_vault, kp) = setup();
        let mut rng = rand::rngs::OsRng;
        let mut pool = NoncePool::new();

        pool.refill(2, &kp, &mut rng);

        assert!(pool.take().is_some());
        assert!(pool.take().is_some());
        assert!(pool.take().is_none(), "exhausted pool must return None");
    }

    #[test]
    fn test_pool_refill_appends() {
        let (_vault, kp) = setup();
        let mut rng = rand::rngs::OsRng;
        let mut pool = NoncePool::new();

        pool.refill(2, &kp, &mut rng);
        pool.refill(3, &kp, &mut rng);
        assert_eq!(pool.len(), 5, "refill must append");
    }
}

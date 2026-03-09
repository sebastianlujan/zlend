// RFC-OGB-001 §8: Oblivious Chain Sync + NullifierWatch
//
// The Relayer monitors the Zcash chain for vault-related events without
// learning note details. The Signer's NullifierWatch service monitors
// opaque 32-byte nullifier values to detect vault spends.
//
// Key principle: the Relayer sees encrypted Actions and can trial-decrypt
// with the vault's IVK. The Signer watches nullifiers without knowing
// which notes they correspond to.

use blake2b_simd::Params as Blake2bParams;
use std::collections::{HashMap, HashSet};

// ---------------------------------------------------------------------------
// Domain separators
// ---------------------------------------------------------------------------

const PERSONAL_NULLIFIER_TAG: &[u8; 16] = b"OGBank_NullTag__";

// ---------------------------------------------------------------------------
// NullifierWatch (§8.2)
// ---------------------------------------------------------------------------

/// A nullifier being watched by the Signer.
///
/// The Signer receives opaque 32-byte nullifier values from the vault's
/// notes without learning the note contents, amounts, or recipients.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WatchedNullifier {
    /// The raw 32-byte nullifier from the Orchard note.
    pub nullifier: [u8; 32],
    /// Opaque tag for correlation (e.g. vault_id prefix).
    pub tag: [u8; 32],
}

/// NullifierWatch service — monitors the chain for spent nullifiers (§8.2).
///
/// The Signer registers nullifiers from vault notes. When the chain reveals
/// a spent nullifier, the Signer detects it and can trigger vault state
/// transitions (e.g., ACTIVE → FROZEN if unauthorized spend detected).
pub struct NullifierWatch {
    /// Set of nullifiers we are watching.
    watched: HashMap<[u8; 32], WatchedNullifier>,
    /// Nullifiers that have been detected as spent on-chain.
    detected_spent: HashSet<[u8; 32]>,
}

impl NullifierWatch {
    pub fn new() -> Self {
        Self {
            watched: HashMap::new(),
            detected_spent: HashSet::new(),
        }
    }

    /// Register a nullifier to watch.
    pub fn watch(&mut self, nullifier: [u8; 32], tag: [u8; 32]) {
        self.watched.insert(
            nullifier,
            WatchedNullifier { nullifier, tag },
        );
    }

    /// Remove a nullifier from the watch set.
    pub fn unwatch(&mut self, nullifier: &[u8; 32]) {
        self.watched.remove(nullifier);
    }

    /// Check a set of on-chain nullifiers against the watch set.
    /// Returns any watched nullifiers that appear in the on-chain set.
    pub fn scan_block_nullifiers(
        &mut self,
        on_chain_nullifiers: &[[u8; 32]],
    ) -> Vec<WatchedNullifier> {
        let mut matches = Vec::new();
        for nf in on_chain_nullifiers {
            if let Some(watched) = self.watched.get(nf) {
                if self.detected_spent.insert(*nf) {
                    matches.push(watched.clone());
                }
            }
        }
        matches
    }

    /// Number of nullifiers being watched.
    pub fn watch_count(&self) -> usize {
        self.watched.len()
    }

    /// Number of watched nullifiers detected as spent.
    pub fn spent_count(&self) -> usize {
        self.detected_spent.len()
    }

    /// Check if a specific nullifier has been detected as spent.
    pub fn is_spent(&self, nullifier: &[u8; 32]) -> bool {
        self.detected_spent.contains(nullifier)
    }
}

impl Default for NullifierWatch {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Oblivious Sync State (§8.1)
// ---------------------------------------------------------------------------

/// Vault lifecycle states per RFC-OGB-001 §9.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VaultState {
    /// Vault created, no funds deposited yet.
    Inactive,
    /// Funds deposited and confirmed, vault operational.
    Active,
    /// Authorization tickets issued, delegation in effect.
    Delegated,
    /// Delegation period ending, renewal window open.
    Renewal,
    /// Unauthorized spend detected or security event.
    Frozen,
    /// All funds swept to recovery address.
    Swept,
}

impl VaultState {
    /// Convert to string for DB storage.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Inactive => "inactive",
            Self::Active => "active",
            Self::Delegated => "delegated",
            Self::Renewal => "renewal",
            Self::Frozen => "frozen",
            Self::Swept => "swept",
        }
    }

    /// Parse from DB string.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "inactive" => Some(Self::Inactive),
            "active" => Some(Self::Active),
            "delegated" => Some(Self::Delegated),
            "renewal" => Some(Self::Renewal),
            "frozen" => Some(Self::Frozen),
            "swept" => Some(Self::Swept),
            _ => None,
        }
    }

    /// Returns the set of valid next states from the current state (RFC §9.1).
    pub fn valid_transitions(&self) -> &'static [VaultState] {
        match self {
            Self::Inactive => &[Self::Active],
            Self::Active => &[Self::Delegated, Self::Frozen, Self::Swept],
            Self::Delegated => &[Self::Active, Self::Renewal, Self::Frozen, Self::Swept],
            Self::Renewal => &[Self::Delegated, Self::Frozen, Self::Swept],
            Self::Frozen => &[Self::Swept],
            Self::Swept => &[],
        }
    }

    /// Check if transition to `target` is valid.
    pub fn can_transition_to(&self, target: &VaultState) -> bool {
        self.valid_transitions().contains(target)
    }
}

/// Compact block event relevant to a vault.
#[derive(Clone, Debug)]
pub enum VaultEvent {
    /// New note received (deposit or change).
    NoteReceived {
        /// Note value in zatoshi.
        value: u64,
        /// Position in the commitment tree.
        position: u64,
    },
    /// A watched nullifier was spent on-chain.
    NullifierSpent {
        /// The spent nullifier.
        nullifier: [u8; 32],
        /// The tag associated with this nullifier.
        tag: [u8; 32],
    },
    /// Block height advanced.
    BlockAdvanced {
        height: u32,
    },
}

/// Oblivious sync tracker for a single vault (§8.1).
///
/// Tracks vault balance, state, and processes chain events without
/// requiring the Signer to know note details.
pub struct VaultSyncState {
    /// Vault identifier.
    pub vault_id: [u8; 32],
    /// Current vault lifecycle state.
    pub state: VaultState,
    /// Known balance in zatoshi (from trial decryption).
    pub balance: u64,
    /// Last scanned block height.
    pub last_scanned_height: u32,
    /// Number of notes received.
    pub notes_received: u64,
    /// Number of authorized spends detected.
    pub authorized_spends: u64,
    /// Number of unauthorized spends detected (triggers FROZEN).
    pub unauthorized_spends: u64,
    /// Set of nullifiers expected from authorized spends.
    authorized_nullifiers: HashSet<[u8; 32]>,
}

impl VaultSyncState {
    /// Create a new sync state for a vault.
    pub fn new(vault_id: [u8; 32]) -> Self {
        Self {
            vault_id,
            state: VaultState::Inactive,
            balance: 0,
            last_scanned_height: 0,
            notes_received: 0,
            authorized_spends: 0,
            unauthorized_spends: 0,
            authorized_nullifiers: HashSet::new(),
        }
    }

    /// Register a nullifier as expected from an authorized spend.
    pub fn register_authorized_nullifier(&mut self, nullifier: [u8; 32]) {
        self.authorized_nullifiers.insert(nullifier);
    }

    /// Process a vault event and update state accordingly.
    pub fn process_event(&mut self, event: &VaultEvent) {
        match event {
            VaultEvent::NoteReceived { value, .. } => {
                self.balance = self.balance.saturating_add(*value);
                self.notes_received += 1;
                if self.state == VaultState::Inactive {
                    self.state = VaultState::Active;
                }
            }
            VaultEvent::NullifierSpent { nullifier, .. } => {
                if self.authorized_nullifiers.remove(nullifier) {
                    // Expected spend (authorized via ticket)
                    self.authorized_spends += 1;
                } else {
                    // Unexpected spend — freeze the vault
                    self.unauthorized_spends += 1;
                    self.state = VaultState::Frozen;
                }
            }
            VaultEvent::BlockAdvanced { height } => {
                self.last_scanned_height = *height;
            }
        }
    }

    /// Transition vault to Delegated state (tickets issued).
    pub fn delegate(&mut self) -> Result<(), &'static str> {
        match self.state {
            VaultState::Active => {
                self.state = VaultState::Delegated;
                Ok(())
            }
            _ => Err("can only delegate from Active state"),
        }
    }

    /// Transition vault to Renewal state (delegation period ending, RFC §9.1).
    pub fn enter_renewal(&mut self) -> Result<(), &'static str> {
        match self.state {
            VaultState::Delegated => {
                self.state = VaultState::Renewal;
                Ok(())
            }
            _ => Err("can only enter renewal from Delegated state"),
        }
    }

    /// Complete renewal — return to Delegated with new ticket batch (RFC §9.1).
    pub fn complete_renewal(&mut self) -> Result<(), &'static str> {
        match self.state {
            VaultState::Renewal => {
                self.state = VaultState::Delegated;
                Ok(())
            }
            _ => Err("can only complete renewal from Renewal state"),
        }
    }

    /// Explicitly freeze the vault (for revocation, RFC §9.2).
    pub fn freeze(&mut self) -> Result<(), &'static str> {
        match self.state {
            VaultState::Active | VaultState::Delegated | VaultState::Renewal => {
                self.state = VaultState::Frozen;
                Ok(())
            }
            VaultState::Frozen => Err("vault already frozen"),
            _ => Err("cannot freeze from current state"),
        }
    }

    /// Transition vault to Swept state (funds recovered).
    pub fn sweep(&mut self) -> Result<(), &'static str> {
        self.state = VaultState::Swept;
        self.balance = 0;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Nullifier tag computation
// ---------------------------------------------------------------------------

/// Compute an opaque tag for a nullifier.
///
/// `tag = BLAKE2b-256("OGBank_NullTag__", vault_id || nullifier)`
///
/// Used to correlate nullifiers to vaults without revealing note details.
pub fn compute_nullifier_tag(vault_id: &[u8; 32], nullifier: &[u8; 32]) -> [u8; 32] {
    let mut out = [0u8; 32];
    let hash = Blake2bParams::new()
        .hash_length(32)
        .personal(PERSONAL_NULLIFIER_TAG)
        .to_state()
        .update(vault_id)
        .update(nullifier)
        .finalize();
    out.copy_from_slice(hash.as_bytes());
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- NullifierWatch tests ---

    #[test]
    fn test_watch_and_detect() {
        let mut watch = NullifierWatch::new();
        let nf = [1u8; 32];
        let tag = [0xAA; 32];

        watch.watch(nf, tag);
        assert_eq!(watch.watch_count(), 1);
        assert!(!watch.is_spent(&nf));

        // Simulate block with this nullifier
        let matches = watch.scan_block_nullifiers(&[nf]);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].nullifier, nf);
        assert_eq!(matches[0].tag, tag);
        assert!(watch.is_spent(&nf));
        assert_eq!(watch.spent_count(), 1);
    }

    #[test]
    fn test_watch_no_match() {
        let mut watch = NullifierWatch::new();
        watch.watch([1u8; 32], [0u8; 32]);

        // Block with different nullifier
        let matches = watch.scan_block_nullifiers(&[[2u8; 32]]);
        assert!(matches.is_empty());
        assert_eq!(watch.spent_count(), 0);
    }

    #[test]
    fn test_watch_duplicate_detection_ignored() {
        let mut watch = NullifierWatch::new();
        let nf = [1u8; 32];
        watch.watch(nf, [0u8; 32]);

        // First scan detects it
        let m1 = watch.scan_block_nullifiers(&[nf]);
        assert_eq!(m1.len(), 1);

        // Second scan: same nullifier, but already detected
        let m2 = watch.scan_block_nullifiers(&[nf]);
        assert!(m2.is_empty());
        assert_eq!(watch.spent_count(), 1);
    }

    #[test]
    fn test_watch_multiple_nullifiers() {
        let mut watch = NullifierWatch::new();
        let nf1 = [1u8; 32];
        let nf2 = [2u8; 32];
        let nf3 = [3u8; 32];

        watch.watch(nf1, [0xA; 32]);
        watch.watch(nf2, [0xB; 32]);
        // nf3 is NOT watched

        let matches = watch.scan_block_nullifiers(&[nf1, nf2, nf3]);
        assert_eq!(matches.len(), 2);
        assert_eq!(watch.spent_count(), 2);
    }

    #[test]
    fn test_unwatch() {
        let mut watch = NullifierWatch::new();
        let nf = [1u8; 32];
        watch.watch(nf, [0u8; 32]);
        assert_eq!(watch.watch_count(), 1);

        watch.unwatch(&nf);
        assert_eq!(watch.watch_count(), 0);

        // Should not detect anymore
        let matches = watch.scan_block_nullifiers(&[nf]);
        assert!(matches.is_empty());
    }

    // --- VaultSyncState tests ---

    #[test]
    fn test_vault_starts_inactive() {
        let vault = VaultSyncState::new([0u8; 32]);
        assert_eq!(vault.state, VaultState::Inactive);
        assert_eq!(vault.balance, 0);
    }

    #[test]
    fn test_note_received_activates_vault() {
        let mut vault = VaultSyncState::new([0u8; 32]);
        vault.process_event(&VaultEvent::NoteReceived {
            value: 1_000_000,
            position: 0,
        });
        assert_eq!(vault.state, VaultState::Active);
        assert_eq!(vault.balance, 1_000_000);
        assert_eq!(vault.notes_received, 1);
    }

    #[test]
    fn test_multiple_deposits_accumulate() {
        let mut vault = VaultSyncState::new([0u8; 32]);
        vault.process_event(&VaultEvent::NoteReceived {
            value: 500_000,
            position: 0,
        });
        vault.process_event(&VaultEvent::NoteReceived {
            value: 300_000,
            position: 1,
        });
        assert_eq!(vault.balance, 800_000);
        assert_eq!(vault.notes_received, 2);
    }

    #[test]
    fn test_authorized_spend_tracked() {
        let mut vault = VaultSyncState::new([0u8; 32]);
        vault.state = VaultState::Delegated;

        let nf = [5u8; 32];
        vault.register_authorized_nullifier(nf);

        vault.process_event(&VaultEvent::NullifierSpent {
            nullifier: nf,
            tag: [0u8; 32],
        });

        assert_eq!(vault.authorized_spends, 1);
        assert_eq!(vault.unauthorized_spends, 0);
        assert_eq!(vault.state, VaultState::Delegated); // NOT frozen
    }

    #[test]
    fn test_unauthorized_spend_freezes_vault() {
        let mut vault = VaultSyncState::new([0u8; 32]);
        vault.state = VaultState::Active;

        let nf = [5u8; 32]; // NOT registered as authorized

        vault.process_event(&VaultEvent::NullifierSpent {
            nullifier: nf,
            tag: [0u8; 32],
        });

        assert_eq!(vault.unauthorized_spends, 1);
        assert_eq!(vault.state, VaultState::Frozen);
    }

    #[test]
    fn test_block_advanced_updates_height() {
        let mut vault = VaultSyncState::new([0u8; 32]);
        vault.process_event(&VaultEvent::BlockAdvanced { height: 1000 });
        assert_eq!(vault.last_scanned_height, 1000);
    }

    #[test]
    fn test_delegate_from_active() {
        let mut vault = VaultSyncState::new([0u8; 32]);
        vault.state = VaultState::Active;
        assert!(vault.delegate().is_ok());
        assert_eq!(vault.state, VaultState::Delegated);
    }

    #[test]
    fn test_delegate_from_inactive_fails() {
        let mut vault = VaultSyncState::new([0u8; 32]);
        assert!(vault.delegate().is_err());
    }

    #[test]
    fn test_sweep_clears_balance() {
        let mut vault = VaultSyncState::new([0u8; 32]);
        vault.balance = 5_000_000;
        vault.state = VaultState::Frozen;
        assert!(vault.sweep().is_ok());
        assert_eq!(vault.state, VaultState::Swept);
        assert_eq!(vault.balance, 0);
    }

    // --- Nullifier tag tests ---

    #[test]
    fn test_nullifier_tag_deterministic() {
        let vault_id = [1u8; 32];
        let nf = [2u8; 32];
        assert_eq!(
            compute_nullifier_tag(&vault_id, &nf),
            compute_nullifier_tag(&vault_id, &nf)
        );
    }

    #[test]
    fn test_nullifier_tag_different_vaults() {
        let nf = [2u8; 32];
        let t1 = compute_nullifier_tag(&[1u8; 32], &nf);
        let t2 = compute_nullifier_tag(&[3u8; 32], &nf);
        assert_ne!(t1, t2);
    }

    #[test]
    fn test_nullifier_tag_different_nullifiers() {
        let vault_id = [1u8; 32];
        let t1 = compute_nullifier_tag(&vault_id, &[2u8; 32]);
        let t2 = compute_nullifier_tag(&vault_id, &[3u8; 32]);
        assert_ne!(t1, t2);
    }

    // --- VaultState method tests ---

    #[test]
    fn test_vault_state_string_roundtrip() {
        let states = [
            VaultState::Inactive,
            VaultState::Active,
            VaultState::Delegated,
            VaultState::Renewal,
            VaultState::Frozen,
            VaultState::Swept,
        ];
        for state in &states {
            let s = state.as_str();
            let parsed = VaultState::parse(s);
            assert_eq!(parsed, Some(*state), "roundtrip failed for {s}");
        }
    }

    #[test]
    fn test_vault_state_parse_unknown() {
        assert!(VaultState::parse("unknown").is_none());
        assert!(VaultState::parse("").is_none());
    }

    #[test]
    fn test_vault_state_transition_graph() {
        // Inactive -> Active only
        assert!(VaultState::Inactive.can_transition_to(&VaultState::Active));
        assert!(!VaultState::Inactive.can_transition_to(&VaultState::Delegated));
        assert!(!VaultState::Inactive.can_transition_to(&VaultState::Frozen));

        // Active -> Delegated, Frozen, Swept
        assert!(VaultState::Active.can_transition_to(&VaultState::Delegated));
        assert!(VaultState::Active.can_transition_to(&VaultState::Frozen));
        assert!(VaultState::Active.can_transition_to(&VaultState::Swept));
        assert!(!VaultState::Active.can_transition_to(&VaultState::Inactive));

        // Delegated -> Active, Renewal, Frozen, Swept
        assert!(VaultState::Delegated.can_transition_to(&VaultState::Active));
        assert!(VaultState::Delegated.can_transition_to(&VaultState::Renewal));
        assert!(VaultState::Delegated.can_transition_to(&VaultState::Frozen));
        assert!(VaultState::Delegated.can_transition_to(&VaultState::Swept));

        // Renewal -> Delegated, Frozen, Swept
        assert!(VaultState::Renewal.can_transition_to(&VaultState::Delegated));
        assert!(VaultState::Renewal.can_transition_to(&VaultState::Frozen));
        assert!(!VaultState::Renewal.can_transition_to(&VaultState::Active));

        // Frozen -> Swept only
        assert!(VaultState::Frozen.can_transition_to(&VaultState::Swept));
        assert!(!VaultState::Frozen.can_transition_to(&VaultState::Active));

        // Swept -> nothing
        assert!(VaultState::Swept.valid_transitions().is_empty());
    }

    #[test]
    fn test_vault_renewal_flow() {
        let mut vault = VaultSyncState::new([0u8; 32]);
        vault.state = VaultState::Delegated;

        // Enter renewal
        assert!(vault.enter_renewal().is_ok());
        assert_eq!(vault.state, VaultState::Renewal);

        // Complete renewal (back to Delegated with new tickets)
        assert!(vault.complete_renewal().is_ok());
        assert_eq!(vault.state, VaultState::Delegated);
    }

    #[test]
    fn test_vault_renewal_invalid_states() {
        let mut vault = VaultSyncState::new([0u8; 32]);
        vault.state = VaultState::Active;

        // Can't enter renewal from Active
        assert!(vault.enter_renewal().is_err());

        // Can't complete renewal from Active
        assert!(vault.complete_renewal().is_err());
    }

    #[test]
    fn test_vault_freeze_explicit() {
        let mut vault = VaultSyncState::new([0u8; 32]);
        vault.state = VaultState::Delegated;

        assert!(vault.freeze().is_ok());
        assert_eq!(vault.state, VaultState::Frozen);

        // Can't freeze again
        assert!(vault.freeze().is_err());
    }

    // --- Integration: NullifierWatch + VaultSyncState ---

    #[test]
    fn test_end_to_end_watch_and_sync() {
        let vault_id = [0xAB; 32];
        let mut vault = VaultSyncState::new(vault_id);
        let mut watch = NullifierWatch::new();

        // Deposit activates vault
        vault.process_event(&VaultEvent::NoteReceived {
            value: 10_000_000,
            position: 0,
        });
        assert_eq!(vault.state, VaultState::Active);

        // Delegate vault
        vault.delegate().unwrap();
        assert_eq!(vault.state, VaultState::Delegated);

        // Register authorized nullifier and watch it
        let auth_nf = [0x01; 32];
        let tag = compute_nullifier_tag(&vault_id, &auth_nf);
        vault.register_authorized_nullifier(auth_nf);
        watch.watch(auth_nf, tag);

        // Simulate block with the authorized nullifier
        let detected = watch.scan_block_nullifiers(&[auth_nf]);
        assert_eq!(detected.len(), 1);

        // Process the spend event
        vault.process_event(&VaultEvent::NullifierSpent {
            nullifier: auth_nf,
            tag,
        });
        assert_eq!(vault.authorized_spends, 1);
        assert_eq!(vault.state, VaultState::Delegated); // Still delegated

        // Now an UNAUTHORIZED nullifier appears
        let rogue_nf = [0xFF; 32];
        watch.watch(rogue_nf, compute_nullifier_tag(&vault_id, &rogue_nf));
        let detected = watch.scan_block_nullifiers(&[rogue_nf]);
        assert_eq!(detected.len(), 1);

        vault.process_event(&VaultEvent::NullifierSpent {
            nullifier: rogue_nf,
            tag: compute_nullifier_tag(&vault_id, &rogue_nf),
        });
        assert_eq!(vault.unauthorized_spends, 1);
        assert_eq!(vault.state, VaultState::Frozen); // FROZEN!
    }
}

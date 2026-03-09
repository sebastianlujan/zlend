// RFC-OGB-001 §10.2: Communication Protocol Message Types
//
// Typed messages for inter-participant communication.
// All messages are serialized with a compact binary framing:
//   [message_type: u8][payload_length: u32_le][payload][hmac: 32 bytes]
//
// §5.6: Ticket batch payloads (RelayerTicketBatch, SignerTicketBatch)
// §6.2: NullifierWatch request/alert
// §7.4: SpendableNoteSet for note provision

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Ticket Batch Payloads (§5.6)
// ---------------------------------------------------------------------------

/// Payload sent to the Relayer after ticket batch creation.
///
/// Contains the auth_secrets (so the Relayer can open tickets when spending),
/// the full tree for proof computation, and spending parameters.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelayerTicketBatch {
    pub tickets: Vec<RelayerTicketEntry>,
    pub auth_tree_root: String,
}

/// A single ticket entry as seen by the Relayer.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelayerTicketEntry {
    /// The auth_secret (shared with Relayer for ticket opening).
    #[serde(with = "hex")]
    pub auth_secret: Vec<u8>,
    /// Maximum zatoshi for this ticket.
    pub max_amount: u64,
    /// H(recipient_address).
    #[serde(with = "hex")]
    pub destination_hash: Vec<u8>,
    /// Block height after which ticket is invalid.
    pub expiry_block: u32,
    /// Pre-generated FROST nonce commitment (hiding || binding) from the Signer.
    #[serde(with = "hex")]
    pub nonce_commitment: Vec<u8>,
}

/// Payload sent to the Signer after ticket batch creation.
///
/// Contains the tree root, valid nullifier hashes for lookup,
/// the nonce registry mapping, and ticket parameters for validation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignerTicketBatch {
    /// The auth tree root for this batch.
    #[serde(with = "hex")]
    pub auth_tree_root: Vec<u8>,
    /// Valid nullifier hashes for all tickets in this batch.
    pub valid_nullifier_hashes: Vec<String>,
    /// Ticket parameters for each ticket (for signer-side validation).
    pub ticket_params: Vec<SignerTicketParams>,
}

/// Ticket parameters as seen by the Signer (no auth_secret — the Signer
/// derives what it needs from the nullifier hash mapping).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignerTicketParams {
    pub max_amount: u64,
    #[serde(with = "hex")]
    pub destination_hash: Vec<u8>,
    pub expiry_block: u32,
}

// ---------------------------------------------------------------------------
// Chain Sync Messages (§6.2)
// ---------------------------------------------------------------------------

/// Request to start watching nullifiers on-chain.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WatchRequest {
    #[serde(with = "hex")]
    pub vault_id: Vec<u8>,
    /// Opaque 32-byte nullifier values to watch.
    pub nullifiers: Vec<String>,
    /// Start monitoring from this block height.
    pub start_block: u32,
}

/// Alert when a watched nullifier is found on-chain.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NullifierAlert {
    #[serde(with = "hex")]
    pub vault_id: Vec<u8>,
    #[serde(with = "hex")]
    pub nullifier: Vec<u8>,
    pub block_height: u32,
    #[serde(with = "hex")]
    pub block_hash: Vec<u8>,
}

// ---------------------------------------------------------------------------
// Note Provision (§7.4)
// ---------------------------------------------------------------------------

/// A note provided by the Signer to the Relayer for an authorized spend.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpendableNote {
    /// Note value in zatoshi.
    pub value_zat: u64,
    /// Pre-computed nullifier for this note.
    #[serde(with = "hex")]
    pub nullifier: Vec<u8>,
    /// Merkle path in the commitment tree (serialized).
    #[serde(with = "hex")]
    pub merkle_path: Vec<u8>,
}

/// Set of spendable notes provided for a signing request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpendableNoteSet {
    pub notes: Vec<SpendableNote>,
    /// Current Orchard anchor (commitment tree root).
    #[serde(with = "hex")]
    pub anchor: Vec<u8>,
    /// Block height of this snapshot.
    pub as_of_block: u32,
}

// ---------------------------------------------------------------------------
// Signing Messages (§7.2–7.4)
// ---------------------------------------------------------------------------

/// Request from Relayer to Signer for spendable notes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpendableNoteRequest {
    #[serde(with = "hex")]
    pub vault_id: Vec<u8>,
    pub required_amount: u64,
}

/// Proof and sighash returned by the Signer after proof generation (§7.2.3).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProofAndSighash {
    /// The Halo2 proof (serialized).
    #[serde(with = "hex")]
    pub proof: Vec<u8>,
    /// The randomized validating key rk.
    #[serde(with = "hex")]
    pub rk: Vec<u8>,
    /// The computed SIGHASH.
    #[serde(with = "hex")]
    pub sighash: Vec<u8>,
    /// The randomizer α (for FROST re-randomized signing).
    #[serde(with = "hex")]
    pub randomizer: Vec<u8>,
}

/// Response from Signer with a signature share.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignatureShareResponse {
    pub success: bool,
    /// Hex-encoded signature share (if success).
    pub signature_share: Option<String>,
    /// Error description (if failure).
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// Lifecycle Messages (§9)
// ---------------------------------------------------------------------------

/// Revocation notice from Owner to Signer/Relayer.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RevocationNotice {
    #[serde(with = "hex")]
    pub vault_id: Vec<u8>,
    /// Owner's signature over the revocation (proves authorization).
    #[serde(with = "hex")]
    pub owner_signature: Vec<u8>,
}

// ---------------------------------------------------------------------------
// Wire Format (§10.3)
// ---------------------------------------------------------------------------

/// Message type discriminants for wire framing.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageType {
    // Key ceremony
    DKGRound1Package = 0x01,
    DKGRound2Package = 0x02,
    DKGComplete = 0x03,

    // Ticket management
    TicketBatchForRelayer = 0x10,
    TicketBatchForSigner = 0x11,

    // Chain sync
    NullifierWatchReq = 0x20,
    NullifierAlertMsg = 0x21,
    NoteReceived = 0x22,

    // Signing
    SpendableNoteReq = 0x30,
    SpendableNoteResp = 0x31,
    UnsignedTxForProof = 0x32,
    ProofAndSighashMsg = 0x33,
    SigningReq = 0x34,
    SignatureShareResp = 0x35,

    // Lifecycle
    Revocation = 0x40,
    ShareRefresh = 0x41,
}

impl MessageType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x01 => Some(Self::DKGRound1Package),
            0x02 => Some(Self::DKGRound2Package),
            0x03 => Some(Self::DKGComplete),
            0x10 => Some(Self::TicketBatchForRelayer),
            0x11 => Some(Self::TicketBatchForSigner),
            0x20 => Some(Self::NullifierWatchReq),
            0x21 => Some(Self::NullifierAlertMsg),
            0x22 => Some(Self::NoteReceived),
            0x30 => Some(Self::SpendableNoteReq),
            0x31 => Some(Self::SpendableNoteResp),
            0x32 => Some(Self::UnsignedTxForProof),
            0x33 => Some(Self::ProofAndSighashMsg),
            0x34 => Some(Self::SigningReq),
            0x35 => Some(Self::SignatureShareResp),
            0x40 => Some(Self::Revocation),
            0x41 => Some(Self::ShareRefresh),
            _ => None,
        }
    }
}

/// Wire-framed message envelope (§10.3).
///
/// Format: [type: u8][length: u32_le][payload: bytes][hmac: 32 bytes]
pub struct WireMessage {
    pub msg_type: MessageType,
    pub payload: Vec<u8>,
    pub hmac: [u8; 32],
}

impl WireMessage {
    /// Encode a message to wire format.
    ///
    /// The `session_key` is used to compute the HMAC over
    /// `msg_type || payload_length || payload`.
    pub fn encode(msg_type: MessageType, payload: &[u8], session_key: &[u8; 32]) -> Vec<u8> {
        use blake2b_simd::Params as Blake2bParams;

        let len = payload.len() as u32;
        let mut buf = Vec::with_capacity(1 + 4 + payload.len() + 32);
        buf.push(msg_type as u8);
        buf.extend_from_slice(&len.to_le_bytes());
        buf.extend_from_slice(payload);

        // HMAC: keyed BLAKE2b-256 over type || length || payload
        let hmac = Blake2bParams::new()
            .hash_length(32)
            .key(session_key)
            .to_state()
            .update(&buf)
            .finalize();
        buf.extend_from_slice(hmac.as_bytes());

        buf
    }

    /// Decode and verify a wire message.
    ///
    /// Returns `None` if the message is malformed or the HMAC doesn't verify.
    pub fn decode(data: &[u8], session_key: &[u8; 32]) -> Option<Self> {
        use blake2b_simd::Params as Blake2bParams;

        // Minimum: 1 (type) + 4 (length) + 0 (payload) + 32 (hmac) = 37
        if data.len() < 37 {
            return None;
        }

        let msg_type = MessageType::from_u8(data[0])?;
        let len = u32::from_le_bytes([data[1], data[2], data[3], data[4]]) as usize;

        // Check total length: 1 + 4 + len + 32
        if data.len() != 5 + len + 32 {
            return None;
        }

        let payload = &data[5..5 + len];
        let received_hmac = &data[5 + len..5 + len + 32];

        // Verify HMAC
        let expected = Blake2bParams::new()
            .hash_length(32)
            .key(session_key)
            .to_state()
            .update(&data[..5 + len])
            .finalize();

        if expected.as_bytes() != received_hmac {
            return None;
        }

        let mut hmac = [0u8; 32];
        hmac.copy_from_slice(received_hmac);

        Some(Self {
            msg_type,
            payload: payload.to_vec(),
            hmac,
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type_roundtrip() {
        for &mt in &[
            MessageType::DKGRound1Package,
            MessageType::TicketBatchForRelayer,
            MessageType::NullifierWatchReq,
            MessageType::SpendableNoteReq,
            MessageType::SigningReq,
            MessageType::Revocation,
        ] {
            let v = mt as u8;
            let decoded = MessageType::from_u8(v);
            assert_eq!(decoded, Some(mt), "roundtrip for {v:#x}");
        }
    }

    #[test]
    fn test_message_type_unknown() {
        assert!(MessageType::from_u8(0xFF).is_none());
        assert!(MessageType::from_u8(0x00).is_none());
    }

    #[test]
    fn test_wire_encode_decode() {
        let session_key = [99u8; 32];
        let payload = b"test payload data";

        let encoded = WireMessage::encode(
            MessageType::SigningReq,
            payload,
            &session_key,
        );

        let decoded = WireMessage::decode(&encoded, &session_key);
        assert!(decoded.is_some(), "decode must succeed");

        let msg = decoded.unwrap();
        assert_eq!(msg.msg_type, MessageType::SigningReq);
        assert_eq!(msg.payload, payload);
    }

    #[test]
    fn test_wire_wrong_key_rejected() {
        let session_key = [99u8; 32];
        let wrong_key = [88u8; 32];
        let payload = b"secret";

        let encoded = WireMessage::encode(MessageType::Revocation, payload, &session_key);

        let decoded = WireMessage::decode(&encoded, &wrong_key);
        assert!(decoded.is_none(), "wrong key must reject message");
    }

    #[test]
    fn test_wire_tampered_payload_rejected() {
        let session_key = [99u8; 32];
        let payload = b"original";

        let mut encoded = WireMessage::encode(MessageType::NullifierAlertMsg, payload, &session_key);

        // Tamper with payload byte
        encoded[6] ^= 0xFF;

        let decoded = WireMessage::decode(&encoded, &session_key);
        assert!(decoded.is_none(), "tampered message must be rejected");
    }

    #[test]
    fn test_wire_truncated_rejected() {
        let session_key = [99u8; 32];
        let encoded = WireMessage::encode(MessageType::DKGComplete, b"data", &session_key);

        // Truncate
        let decoded = WireMessage::decode(&encoded[..10], &session_key);
        assert!(decoded.is_none(), "truncated message must be rejected");
    }

    #[test]
    fn test_wire_empty_payload() {
        let session_key = [1u8; 32];
        let encoded = WireMessage::encode(MessageType::DKGRound1Package, &[], &session_key);

        let msg = WireMessage::decode(&encoded, &session_key).unwrap();
        assert!(msg.payload.is_empty());
        assert_eq!(msg.msg_type, MessageType::DKGRound1Package);
    }

    #[test]
    fn test_relayer_ticket_batch_serde() {
        let batch = RelayerTicketBatch {
            tickets: vec![RelayerTicketEntry {
                auth_secret: vec![1u8; 32],
                max_amount: 1_000_000,
                destination_hash: vec![2u8; 32],
                expiry_block: 500_000,
                nonce_commitment: vec![3u8; 64],
            }],
            auth_tree_root: hex::encode([4u8; 32]),
        };

        let json = serde_json::to_string(&batch).unwrap();
        let decoded: RelayerTicketBatch = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.tickets.len(), 1);
        assert_eq!(decoded.tickets[0].max_amount, 1_000_000);
        assert_eq!(decoded.tickets[0].auth_secret, vec![1u8; 32]);
    }

    #[test]
    fn test_signer_ticket_batch_serde() {
        let batch = SignerTicketBatch {
            auth_tree_root: vec![5u8; 32],
            valid_nullifier_hashes: vec![hex::encode([6u8; 32])],
            ticket_params: vec![SignerTicketParams {
                max_amount: 2_000_000,
                destination_hash: vec![7u8; 32],
                expiry_block: 600_000,
            }],
        };

        let json = serde_json::to_string(&batch).unwrap();
        let decoded: SignerTicketBatch = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.ticket_params.len(), 1);
        assert_eq!(decoded.ticket_params[0].max_amount, 2_000_000);
    }

    #[test]
    fn test_spendable_note_set_serde() {
        let set = SpendableNoteSet {
            notes: vec![SpendableNote {
                value_zat: 5_000_000,
                nullifier: vec![8u8; 32],
                merkle_path: vec![9u8; 128],
            }],
            anchor: vec![10u8; 32],
            as_of_block: 100_000,
        };

        let json = serde_json::to_string(&set).unwrap();
        let decoded: SpendableNoteSet = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.notes.len(), 1);
        assert_eq!(decoded.notes[0].value_zat, 5_000_000);
        assert_eq!(decoded.as_of_block, 100_000);
    }

    #[test]
    fn test_watch_request_serde() {
        let req = WatchRequest {
            vault_id: vec![11u8; 32],
            nullifiers: vec![hex::encode([12u8; 32]), hex::encode([13u8; 32])],
            start_block: 50_000,
        };

        let json = serde_json::to_string(&req).unwrap();
        let decoded: WatchRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.nullifiers.len(), 2);
        assert_eq!(decoded.start_block, 50_000);
    }

    #[test]
    fn test_signature_share_response_success() {
        let resp = SignatureShareResponse {
            success: true,
            signature_share: Some(hex::encode([14u8; 32])),
            error: None,
        };

        let json = serde_json::to_string(&resp).unwrap();
        let decoded: SignatureShareResponse = serde_json::from_str(&json).unwrap();
        assert!(decoded.success);
        assert!(decoded.signature_share.is_some());
        assert!(decoded.error.is_none());
    }

    #[test]
    fn test_signature_share_response_failure() {
        let resp = SignatureShareResponse {
            success: false,
            signature_share: None,
            error: Some("TicketExpired".to_string()),
        };

        let json = serde_json::to_string(&resp).unwrap();
        let decoded: SignatureShareResponse = serde_json::from_str(&json).unwrap();
        assert!(!decoded.success);
        assert!(decoded.error.as_ref().unwrap().contains("Expired"));
    }
}

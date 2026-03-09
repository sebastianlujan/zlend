// RFC-OGB-001 §7.3: Automated Signer Daemon
//
// Holds KeyPackage₁ + FVK. Validates signing requests against the
// authorization ticket tree (7 deterministic checks) and produces
// FROST re-randomized signature shares.
//
// Endpoints:
//   GET  /health             — liveness check
//   POST /sign               — validate + produce signature share
//   POST /ceremony           — full local ceremony simulation (MVP demo)
//   GET  /state              — current signer state summary
//   POST /nonce-commit       — generate FROST nonce commitments (Round 1, §7.2.4)
//   POST /sign-share         — produce FROST signature share (Round 2, §7.2.4)
//   POST /nonce-clear        — clear nonce registry (for revocation, §9.2)

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

use ogbank_core::auth::{AuthTree, AuthorizationTicket};
use ogbank_core::frost::{self, Identifier, VaultKeyShares};
use ogbank_core::nonces::NonceRegistry;
use ogbank_core::protocol::{
    DistributedSignRequest, DistributedSignResponse, NonceCommitmentRequest,
    NonceCommitmentResponse,
};
use ogbank_core::signer::{
    compute_sighash, AuthorizationProof, ProposedTx, SignerState, SigningRequest,
};
use reddsa::frost::redpallas::{round1, round2, PallasBlake2b512, SigningPackage};

type NonceCommitment = frost_rerandomized::frost_core::frost::round1::NonceCommitment<PallasBlake2b512>;

// ---------------------------------------------------------------------------
// Application state
// ---------------------------------------------------------------------------

pub struct SignerAppState {
    /// Signer validation state (auth tree root + spent nullifiers + block height).
    pub signer: Mutex<SignerState>,
    /// Vault key shares (MVP: holds all 3 for local ceremony simulation).
    pub vault: VaultKeyShares,
    /// Vault ID (hex).
    pub vault_id: String,
    /// Nonce registry binding pre-committed nonces to ticket nullifiers (§7.2.1).
    pub nonce_registry: Mutex<NonceRegistry>,
    /// This signer's FROST participant ID (1=OwnerA, 2=OwnerB, 3=Relayer).
    pub participant_id: u16,
}

pub fn router(state: Arc<SignerAppState>) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/sign", post(sign_handler))
        .route("/ceremony", post(ceremony_handler))
        .route("/state", get(state_handler))
        .route("/nonce-commit", post(nonce_commit_handler))
        .route("/sign-share", post(sign_share_handler))
        .route("/nonce-clear", post(nonce_clear_handler))
        .with_state(state)
}

// ---------------------------------------------------------------------------
// Health
// ---------------------------------------------------------------------------

async fn health_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({ "status": "ok", "service": "ogbank-signer" })),
    )
}

// ---------------------------------------------------------------------------
// State summary
// ---------------------------------------------------------------------------

async fn state_handler(State(state): State<Arc<SignerAppState>>) -> impl IntoResponse {
    let signer = state.signer.lock().unwrap();
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "vault_id": state.vault_id,
            "auth_tree_root": hex::encode(signer.auth_tree_root),
            "current_block_height": signer.current_block_height,
            "spent_nullifier_count": signer.spent_nullifiers.len(),
        })),
    )
}

// ---------------------------------------------------------------------------
// Sign — validate + produce FROST share (RFC §7.3)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct SignRequest {
    #[serde(with = "hex")]
    pub auth_secret: Vec<u8>,
    pub max_amount: u64,
    pub destination: String,
    pub expiry_block: u32,
    pub amount: u64,
}

async fn sign_handler(
    State(state): State<Arc<SignerAppState>>,
    Json(req): Json<SignRequest>,
) -> impl IntoResponse {
    let auth_secret: [u8; 32] = match req.auth_secret.try_into() {
        Ok(s) => s,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "auth_secret must be 32 bytes" })),
            );
        }
    };

    let dest_bytes = req.destination.as_bytes();
    let ticket = AuthorizationTicket::new(auth_secret, req.max_amount, dest_bytes, req.expiry_block);

    // Build tree + proof for this ticket
    let mut tree = AuthTree::new();
    let leaf_index = tree.insert(ticket.commitment()).unwrap();
    let merkle_proof = tree.proof(leaf_index).unwrap();
    let tree_root = tree.root();

    // Build signing request
    let tx_data = format!("signer-tx,amount={}", req.amount).into_bytes();
    let sighash = compute_sighash(&tx_data);

    let signing_request = SigningRequest {
        sighash,
        auth_proof: AuthorizationProof {
            auth_secret: ticket.auth_secret,
            auth_nullifier_hash: ticket.nullifier_hash(),
            max_amount: ticket.max_amount,
            destination_hash: ticket.destination_hash,
            expiry_block: ticket.expiry_block,
            merkle_proof,
        },
        proposed_tx: ProposedTx {
            total_spend_value: req.amount,
            recipient_address: dest_bytes.to_vec(),
            tx_data,
        },
    };

    // Validate against signer state
    let signer = state.signer.lock().unwrap();

    // For MVP: create a fresh signer state with this ticket's tree root
    // (in production, the signer maintains persistent state across requests)
    let signer_state = SignerState::new(tree_root, signer.current_block_height);

    match signer_state.validate(&signing_request) {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "vault_id": state.vault_id,
                "valid": true,
                "sighash": hex::encode(sighash),
                "nullifier_hash": hex::encode(ticket.nullifier_hash()),
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "vault_id": state.vault_id,
                "valid": false,
                "error": format!("{e:?}"),
            })),
        ),
    }
}

// ---------------------------------------------------------------------------
// Ceremony — full local simulation (MVP demo, RFC §7.2)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CeremonyRequest {
    #[serde(with = "hex")]
    pub auth_secret: Vec<u8>,
    pub max_amount: u64,
    pub destination: String,
    pub expiry_block: u32,
    pub amount: u64,
}

async fn ceremony_handler(
    State(state): State<Arc<SignerAppState>>,
    Json(req): Json<CeremonyRequest>,
) -> impl IntoResponse {
    use ogbank_core::ceremony;

    let auth_secret: [u8; 32] = match req.auth_secret.try_into() {
        Ok(s) => s,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "auth_secret must be 32 bytes" })),
            );
        }
    };

    let dest_bytes = req.destination.as_bytes();
    let ticket = AuthorizationTicket::new(auth_secret, req.max_amount, dest_bytes, req.expiry_block);

    let mut tree = AuthTree::new();
    let leaf_index = tree.insert(ticket.commitment()).unwrap();
    let merkle_proof = tree.proof(leaf_index).unwrap();
    let tree_root = tree.root();

    let tx_data = format!("ceremony-tx,amount={}", req.amount).into_bytes();
    let sighash = compute_sighash(&tx_data);

    let signing_request = SigningRequest {
        sighash,
        auth_proof: AuthorizationProof {
            auth_secret: ticket.auth_secret,
            auth_nullifier_hash: ticket.nullifier_hash(),
            max_amount: ticket.max_amount,
            destination_hash: ticket.destination_hash,
            expiry_block: ticket.expiry_block,
            merkle_proof,
        },
        proposed_tx: ProposedTx {
            total_spend_value: req.amount,
            recipient_address: dest_bytes.to_vec(),
            tx_data,
        },
    };

    let signer = state.signer.lock().unwrap();
    let mut signer_state = SignerState::new(tree_root, signer.current_block_height);
    let signer_ids = [
        Identifier::try_from(frost::OWNER_A_ID).unwrap(),
        Identifier::try_from(frost::RELAYER_ID).unwrap(),
    ];

    let mut rng = rand::rngs::OsRng;
    match ceremony::run_ceremony_local(
        &state.vault,
        &signer_ids,
        &signing_request,
        &mut signer_state,
        &mut rng,
    ) {
        Ok(result) => {
            let sig_bytes = result.signature.serialize();
            let verified = result
                .randomized_verifying_key
                .verify(&sighash, &result.signature)
                .is_ok();

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "vault_id": state.vault_id,
                    "valid": true,
                    "signature": hex::encode(sig_bytes),
                    "verification": if verified { "passed" } else { "failed" },
                })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "vault_id": state.vault_id,
                "error": format!("{e:?}"),
            })),
        ),
    }
}

// ---------------------------------------------------------------------------
// Nonce Commit — FROST Round 1 (RFC §7.2.4)
// ---------------------------------------------------------------------------

async fn nonce_commit_handler(
    State(state): State<Arc<SignerAppState>>,
    Json(req): Json<NonceCommitmentRequest>,
) -> impl IntoResponse {
    let auth_nullifier_hash: [u8; 32] = match req.auth_nullifier_hash.try_into() {
        Ok(h) => h,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!(NonceCommitmentResponse {
                    success: false,
                    participant_id: state.participant_id,
                    commitments: None,
                    error: Some("auth_nullifier_hash must be 32 bytes".into()),
                })),
            );
        }
    };

    // Get the key package for this signer's participant ID
    let id = match Identifier::try_from(state.participant_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!(NonceCommitmentResponse {
                    success: false,
                    participant_id: state.participant_id,
                    commitments: None,
                    error: Some("invalid participant_id".into()),
                })),
            );
        }
    };

    let key_package = match state.vault.key_packages.get(&id) {
        Some(kp) => kp,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!(NonceCommitmentResponse {
                    success: false,
                    participant_id: state.participant_id,
                    commitments: None,
                    error: Some("no key package for this participant".into()),
                })),
            );
        }
    };

    let mut rng = rand::rngs::OsRng;
    let mut registry = state.nonce_registry.lock().unwrap();
    match registry.register_fresh(auth_nullifier_hash, key_package, &mut rng) {
        Some(commitments) => {
            // Serialize commitments: hiding (32 bytes) || binding (32 bytes)
            let hiding = commitments.hiding().serialize();
            let binding = commitments.binding().serialize();
            let mut commit_bytes = Vec::with_capacity(64);
            commit_bytes.extend_from_slice(hiding.as_ref());
            commit_bytes.extend_from_slice(binding.as_ref());

            (
                StatusCode::OK,
                Json(serde_json::json!(NonceCommitmentResponse {
                    success: true,
                    participant_id: state.participant_id,
                    commitments: Some(hex::encode(&commit_bytes)),
                    error: None,
                })),
            )
        }
        None => (
            StatusCode::CONFLICT,
            Json(serde_json::json!(NonceCommitmentResponse {
                success: false,
                participant_id: state.participant_id,
                commitments: None,
                error: Some("nonce already registered for this nullifier hash".into()),
            })),
        ),
    }
}

// ---------------------------------------------------------------------------
// Sign Share — FROST Round 2 (RFC §7.2.4)
// ---------------------------------------------------------------------------

async fn sign_share_handler(
    State(state): State<Arc<SignerAppState>>,
    Json(req): Json<DistributedSignRequest>,
) -> impl IntoResponse {
    let empty_sighash = vec![0u8; 32];

    // Parse auth_secret
    let auth_secret: [u8; 32] = match req.auth_secret.try_into() {
        Ok(s) => s,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!(DistributedSignResponse {
                    success: false,
                    participant_id: state.participant_id,
                    signature_share: None,
                    sighash: empty_sighash,
                    error: Some("auth_secret must be 32 bytes".into()),
                })),
            );
        }
    };

    let dest_bytes = req.destination.as_bytes();
    let ticket = AuthorizationTicket::new(auth_secret, req.max_amount, dest_bytes, req.expiry_block);
    let auth_nullifier_hash = ticket.nullifier_hash();

    // Build auth tree + proof
    let mut tree = AuthTree::new();
    let leaf_index = tree.insert(ticket.commitment()).unwrap();
    let merkle_proof = tree.proof(leaf_index).unwrap();
    let tree_root = tree.root();

    // Reconstruct sighash from tx_data for CHECK 7
    let sighash = compute_sighash(&req.tx_data);

    // Build signing request
    let signing_request = SigningRequest {
        sighash,
        auth_proof: AuthorizationProof {
            auth_secret: ticket.auth_secret,
            auth_nullifier_hash,
            max_amount: ticket.max_amount,
            destination_hash: ticket.destination_hash,
            expiry_block: ticket.expiry_block,
            merkle_proof,
        },
        proposed_tx: ProposedTx {
            total_spend_value: req.amount,
            recipient_address: dest_bytes.to_vec(),
            tx_data: req.tx_data.clone(),
        },
    };

    // Validate all 7 checks (RFC §7.3)
    let signer = state.signer.lock().unwrap();
    // For MVP: create fresh signer state with this ticket's tree root
    // (in production, the signer maintains persistent state across requests)
    let signer_state = SignerState::new(tree_root, signer.current_block_height);
    drop(signer);

    if let Err(e) = signer_state.validate(&signing_request) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!(DistributedSignResponse {
                success: false,
                participant_id: state.participant_id,
                signature_share: None,
                sighash: sighash.to_vec(),
                error: Some(format!("{e:?}")),
            })),
        );
    }

    // Consume nonce from registry
    let mut registry = state.nonce_registry.lock().unwrap();
    let nonce_entry = match registry.consume(&auth_nullifier_hash) {
        Some(entry) => entry,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!(DistributedSignResponse {
                    success: false,
                    participant_id: state.participant_id,
                    signature_share: None,
                    sighash: sighash.to_vec(),
                    error: Some("no nonce registered for this nullifier hash".into()),
                })),
            );
        }
    };
    drop(registry);

    // Deserialize commitment map from request
    let mut commitment_map: BTreeMap<Identifier, round1::SigningCommitments> = BTreeMap::new();
    for (&pid, commit_hex) in &req.commitments {
        let commit_bytes = match hex::decode(commit_hex) {
            Ok(b) => b,
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!(DistributedSignResponse {
                        success: false,
                        participant_id: state.participant_id,
                        signature_share: None,
                        sighash: sighash.to_vec(),
                        error: Some(format!("invalid commitment hex for participant {pid}")),
                    })),
                );
            }
        };
        if commit_bytes.len() != 64 {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!(DistributedSignResponse {
                    success: false,
                    participant_id: state.participant_id,
                    signature_share: None,
                    sighash: sighash.to_vec(),
                    error: Some(format!("commitment must be 64 bytes for participant {pid}")),
                })),
            );
        }

        let id = match Identifier::try_from(pid) {
            Ok(id) => id,
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!(DistributedSignResponse {
                        success: false,
                        participant_id: state.participant_id,
                        signature_share: None,
                        sighash: sighash.to_vec(),
                        error: Some(format!("invalid participant id {pid}")),
                    })),
                );
            }
        };

        // Deserialize hiding (first 32) and binding (last 32)
        let mut hiding_bytes = [0u8; 32];
        let mut binding_bytes = [0u8; 32];
        hiding_bytes.copy_from_slice(&commit_bytes[..32]);
        binding_bytes.copy_from_slice(&commit_bytes[32..]);

        let hiding = match NonceCommitment::deserialize(hiding_bytes) {
            Ok(h) => h,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!(DistributedSignResponse {
                        success: false,
                        participant_id: state.participant_id,
                        signature_share: None,
                        sighash: sighash.to_vec(),
                        error: Some(format!("invalid hiding commitment: {e}")),
                    })),
                );
            }
        };
        let binding = match NonceCommitment::deserialize(binding_bytes) {
            Ok(b) => b,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!(DistributedSignResponse {
                        success: false,
                        participant_id: state.participant_id,
                        signature_share: None,
                        sighash: sighash.to_vec(),
                        error: Some(format!("invalid binding commitment: {e}")),
                    })),
                );
            }
        };

        commitment_map.insert(id, round1::SigningCommitments::new(hiding, binding));
    }

    // Build signing package
    let signing_package = SigningPackage::new(commitment_map, &sighash);

    // Deserialize randomizer point
    let randomizer_point_bytes: [u8; 32] = match req.randomizer_point.try_into() {
        Ok(b) => b,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!(DistributedSignResponse {
                    success: false,
                    participant_id: state.participant_id,
                    signature_share: None,
                    sighash: sighash.to_vec(),
                    error: Some("randomizer_point must be 32 bytes".into()),
                })),
            );
        }
    };

    use frost_rerandomized::frost_core::Group;
    type PallasGroup = <reddsa::frost::redpallas::PallasBlake2b512 as frost_rerandomized::frost_core::Ciphersuite>::Group;

    let randomizer_point = match PallasGroup::deserialize(&randomizer_point_bytes) {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!(DistributedSignResponse {
                    success: false,
                    participant_id: state.participant_id,
                    signature_share: None,
                    sighash: sighash.to_vec(),
                    error: Some(format!("invalid randomizer point: {e}")),
                })),
            );
        }
    };

    // Get this signer's key package
    let signer_id = match Identifier::try_from(state.participant_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!(DistributedSignResponse {
                    success: false,
                    participant_id: state.participant_id,
                    signature_share: None,
                    sighash: sighash.to_vec(),
                    error: Some("invalid participant_id".into()),
                })),
            );
        }
    };
    let key_package = &state.vault.key_packages[&signer_id];

    // Produce FROST re-randomized signature share (Round 2)
    match round2::sign(&signing_package, &nonce_entry.nonces, key_package, &randomizer_point) {
        Ok(share) => {
            let share_bytes: [u8; 32] = share.serialize();
            (
                StatusCode::OK,
                Json(serde_json::json!(DistributedSignResponse {
                    success: true,
                    participant_id: state.participant_id,
                    signature_share: Some(hex::encode(share_bytes)),
                    sighash: sighash.to_vec(),
                    error: None,
                })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!(DistributedSignResponse {
                success: false,
                participant_id: state.participant_id,
                signature_share: None,
                sighash: sighash.to_vec(),
                error: Some(format!("FROST sign failed: {e}")),
            })),
        ),
    }
}

// ---------------------------------------------------------------------------
// Nonce Clear — revocation support (RFC §9.2)
// ---------------------------------------------------------------------------

async fn nonce_clear_handler(
    State(state): State<Arc<SignerAppState>>,
) -> impl IntoResponse {
    let mut registry = state.nonce_registry.lock().unwrap();
    let count = registry.len();
    registry.clear();

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "cleared": count,
            "status": "ok"
        })),
    )
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use anyhow::Context;

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ogbank_signer=info".into()),
        )
        .init();

    // MVP: generate vault keys on startup (in production, loaded from encrypted storage)
    let mut rng = rand::rngs::OsRng;
    let vault = frost::run_dkg_local(&mut rng)
        .map_err(|e| anyhow::anyhow!("DKG failed: {e}"))?;
    let ak = frost::group_public_key_bytes(&vault.public_key_package);
    let vault_id = frost::compute_vault_id(&ak, 0);

    tracing::info!("vault_id: {}", hex::encode(vault_id));
    tracing::info!("group_public_key: {}", hex::encode(ak));

    // Initialize signer state with empty tree root (will be set per-request in MVP)
    let signer_state = SignerState::new([0u8; 32], 100);

    let participant_id: u16 = std::env::var("SIGNER_PARTICIPANT_ID")
        .unwrap_or_else(|_| "1".to_string())
        .parse()
        .expect("SIGNER_PARTICIPANT_ID must be a u16 (1=OwnerA, 2=OwnerB, 3=Relayer)");

    let state = Arc::new(SignerAppState {
        signer: Mutex::new(signer_state),
        vault,
        vault_id: hex::encode(vault_id),
        nonce_registry: Mutex::new(NonceRegistry::new()),
        participant_id,
    });

    let app = router(state);

    let bind_addr = std::env::var("SIGNER_BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3001".to_string());
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .with_context(|| format!("failed to bind to {bind_addr}"))?;

    tracing::info!("ogbank-signer listening on {bind_addr}");
    axum::serve(listener, app).await.context("server error")?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn test_state() -> Arc<SignerAppState> {
        let mut rng = rand::rngs::OsRng;
        let vault = frost::run_dkg_local(&mut rng).expect("DKG");
        let ak = frost::group_public_key_bytes(&vault.public_key_package);
        let vault_id = frost::compute_vault_id(&ak, 0);

        Arc::new(SignerAppState {
            signer: Mutex::new(SignerState::new([0u8; 32], 100)),
            vault,
            vault_id: hex::encode(vault_id),
            nonce_registry: Mutex::new(NonceRegistry::new()),
            participant_id: 1,
        })
    }

    async fn body_json(body: Body) -> serde_json::Value {
        let bytes = body.collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn test_health() {
        let state = test_state();
        let app = router(state);

        let req = Request::builder()
            .method("GET")
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let json = body_json(resp.into_body()).await;
        assert_eq!(json["status"], "ok");
        assert_eq!(json["service"], "ogbank-signer");
    }

    #[tokio::test]
    async fn test_state_endpoint() {
        let state = test_state();
        let app = router(state.clone());

        let req = Request::builder()
            .method("GET")
            .uri("/state")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let json = body_json(resp.into_body()).await;
        assert_eq!(json["vault_id"], state.vault_id);
        assert_eq!(json["current_block_height"], 100);
        assert_eq!(json["spent_nullifier_count"], 0);
    }

    #[tokio::test]
    async fn test_sign_valid() {
        let state = test_state();
        let app = router(state);

        let auth_secret = [42u8; 32];
        let req = Request::builder()
            .method("POST")
            .uri("/sign")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_string(&serde_json::json!({
                    "auth_secret": hex::encode(auth_secret),
                    "max_amount": 1_000_000u64,
                    "destination": "recipient",
                    "expiry_block": 500_000u32,
                    "amount": 500_000u64,
                }))
                .unwrap(),
            ))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let json = body_json(resp.into_body()).await;
        assert_eq!(json["valid"], true);
        assert!(json["sighash"].as_str().unwrap().len() == 64);
    }

    #[tokio::test]
    async fn test_sign_expired_ticket() {
        let state = test_state();
        let app = router(state);

        let auth_secret = [42u8; 32];
        let req = Request::builder()
            .method("POST")
            .uri("/sign")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_string(&serde_json::json!({
                    "auth_secret": hex::encode(auth_secret),
                    "max_amount": 1_000_000u64,
                    "destination": "recipient",
                    "expiry_block": 50u32,
                    "amount": 500_000u64,
                }))
                .unwrap(),
            ))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let json = body_json(resp.into_body()).await;
        assert_eq!(json["valid"], false);
        assert!(json["error"].as_str().unwrap().contains("TicketExpired"));
    }

    #[tokio::test]
    async fn test_ceremony_valid() {
        let state = test_state();
        let app = router(state);

        let auth_secret = [42u8; 32];
        let req = Request::builder()
            .method("POST")
            .uri("/ceremony")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_string(&serde_json::json!({
                    "auth_secret": hex::encode(auth_secret),
                    "max_amount": 1_000_000u64,
                    "destination": "recipient",
                    "expiry_block": 500_000u32,
                    "amount": 500_000u64,
                }))
                .unwrap(),
            ))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let json = body_json(resp.into_body()).await;
        assert_eq!(json["valid"], true);
        assert_eq!(json["verification"], "passed");
        assert!(!json["signature"].as_str().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_ceremony_amount_exceeded() {
        let state = test_state();
        let app = router(state);

        let auth_secret = [42u8; 32];
        let req = Request::builder()
            .method("POST")
            .uri("/ceremony")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_string(&serde_json::json!({
                    "auth_secret": hex::encode(auth_secret),
                    "max_amount": 100u64,
                    "destination": "recipient",
                    "expiry_block": 500_000u32,
                    "amount": 999_999u64,
                }))
                .unwrap(),
            ))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        // Ceremony should fail at validation step
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let json = body_json(resp.into_body()).await;
        assert!(json["error"].as_str().unwrap().contains("AmountExceeded"));
    }

    // --- Distributed ceremony endpoint tests (RFC §7.2.4) ---

    #[tokio::test]
    async fn test_nonce_commit_success() {
        let state = test_state();
        let app = router(state);

        let nullifier_hash = [42u8; 32];
        let req = Request::builder()
            .method("POST")
            .uri("/nonce-commit")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_string(&serde_json::json!({
                    "vault_id": hex::encode([1u8; 32]),
                    "auth_nullifier_hash": hex::encode(nullifier_hash),
                }))
                .unwrap(),
            ))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let json = body_json(resp.into_body()).await;
        assert_eq!(json["success"], true);
        assert_eq!(json["participant_id"], 1);
        // Commitments = hiding (32 bytes) + binding (32 bytes) = 64 bytes = 128 hex chars
        let commitments = json["commitments"].as_str().unwrap();
        assert_eq!(commitments.len(), 128, "commitments must be 64 bytes (128 hex)");
    }

    #[tokio::test]
    async fn test_nonce_commit_duplicate_rejected() {
        let state = test_state();
        let nullifier_hash = [99u8; 32];

        // First request
        let app = router(state.clone());
        let req = Request::builder()
            .method("POST")
            .uri("/nonce-commit")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_string(&serde_json::json!({
                    "vault_id": hex::encode([1u8; 32]),
                    "auth_nullifier_hash": hex::encode(nullifier_hash),
                }))
                .unwrap(),
            ))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Second request with same nullifier hash
        let app = router(state);
        let req = Request::builder()
            .method("POST")
            .uri("/nonce-commit")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_string(&serde_json::json!({
                    "vault_id": hex::encode([1u8; 32]),
                    "auth_nullifier_hash": hex::encode(nullifier_hash),
                }))
                .unwrap(),
            ))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CONFLICT);

        let json = body_json(resp.into_body()).await;
        assert_eq!(json["success"], false);
    }

    #[tokio::test]
    async fn test_nonce_clear() {
        let state = test_state();

        // Register a nonce first
        let app = router(state.clone());
        let req = Request::builder()
            .method("POST")
            .uri("/nonce-commit")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_string(&serde_json::json!({
                    "vault_id": hex::encode([1u8; 32]),
                    "auth_nullifier_hash": hex::encode([50u8; 32]),
                }))
                .unwrap(),
            ))
            .unwrap();
        app.oneshot(req).await.unwrap();
        assert_eq!(state.nonce_registry.lock().unwrap().len(), 1);

        // Clear
        let app = router(state.clone());
        let req = Request::builder()
            .method("POST")
            .uri("/nonce-clear")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let json = body_json(resp.into_body()).await;
        assert_eq!(json["cleared"], 1);
        assert!(state.nonce_registry.lock().unwrap().is_empty());
    }
}

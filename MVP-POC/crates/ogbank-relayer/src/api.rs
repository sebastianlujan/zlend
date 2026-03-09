// Phase 6: REST API Endpoints
//
// POST /register — store viewing/spending keys for a new position
// POST /scan/:id — trial decrypt a Zcash tx to find deposits (uses ivk)
// GET  /balance/:id — return current collateral balance
// POST /vault/create — register a FROST vault
// POST /vault/:id/sign-request — run signer validation + signing ceremony
//
// Reference: docs/technical/08_mvp.md
//            docs/technical/11_rfc-ogb-001.md §4.3, §7

use std::sync::{Arc, Mutex};

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use rusqlite::Connection;
use serde::Deserialize;

use crate::db;

/// Shared application state.
pub struct AppState {
    pub db: Mutex<Connection>,
    pub sk_passphrase: String,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/register", post(register_handler))
        .route("/scan/{id}", post(scan_handler))
        .route("/balance/{id}", get(balance_handler))
        .route("/vault/create", post(vault_create_handler))
        .route("/vault/{vault_id}/sign-request", post(sign_request_handler))
        .with_state(state)
}

// --- Register ---

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub position_id: String,
    pub address: String,
    #[serde(with = "hex")]
    pub sk: Vec<u8>,
    #[serde(with = "hex")]
    pub fvk: Vec<u8>,
    #[serde(with = "hex")]
    pub ivk: Vec<u8>,
}

async fn register_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> impl IntoResponse {
    // Encrypt the spending key before storing
    let sk_encrypted = match ogbank_core::crypto::encrypt_sk(&req.sk, &state.sk_passphrase) {
        Ok(enc) => enc,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("encryption failed: {e}") })),
            );
        }
    };

    let conn = state.db.lock().unwrap();
    match db::insert_position(
        &conn,
        &req.position_id,
        &req.address,
        &req.fvk,
        &sk_encrypted,
        &req.ivk,
    ) {
        Ok(()) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "position_id": req.position_id,
                "status": "registered"
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": format!("registration failed: {e}") })),
        ),
    }
}

// --- Scan ---

#[derive(Deserialize)]
pub struct ScanRequest {
    pub txid: String,
}

async fn scan_handler(
    State(state): State<Arc<AppState>>,
    Path(position_id): Path<String>,
    Json(req): Json<ScanRequest>,
) -> impl IntoResponse {
    let conn = state.db.lock().unwrap();

    // Get the ivk for this position
    let ivk_bytes = match db::get_position_ivk(&conn, &position_id) {
        Ok(ivk) => ivk,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "position not found" })),
            );
        }
    };

    // TODO: Phase 7 — fetch transaction from Tatum API using req.txid,
    // extract Orchard actions, and perform trial decryption with ivk_bytes.
    //
    // For now, return a placeholder acknowledging the scan request.
    // The full implementation requires:
    //   1. zcash.rs: TatumClient.get_transaction(txid)
    //   2. Parse Orchard actions from the response
    //   3. ogbank_core::scan::scan_compact_actions(ivk, actions)
    //   4. For each found note: db::insert_note + update balance
    let _ = ivk_bytes;
    let _ = req.txid;

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "position_id": position_id,
            "found": false,
            "notes_found": 0,
            "total_value_zat": 0,
            "message": "scan endpoint ready — awaiting Tatum API integration (Phase 7)"
        })),
    )
}

// --- Vault Create (RFC §4.3) ---

#[derive(Deserialize)]
pub struct VaultCreateRequest {
    pub vault_id: String,
    pub vault_address: String,
    #[serde(with = "hex")]
    pub group_public_key: Vec<u8>,
}

async fn vault_create_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<VaultCreateRequest>,
) -> impl IntoResponse {
    let conn = state.db.lock().unwrap();
    match db::insert_vault(&conn, &req.vault_id, &req.vault_address, &req.group_public_key) {
        Ok(()) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "vault_id": req.vault_id,
                "status": "registered"
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": format!("vault registration failed: {e}") })),
        ),
    }
}

// --- Sign Request (RFC §7.2–7.3 — MVP simulation) ---

#[derive(Deserialize)]
pub struct SignRequestBody {
    #[serde(with = "hex")]
    pub auth_secret: Vec<u8>,
    pub max_amount: u64,
    pub destination: String,
    pub expiry_block: u32,
    pub current_block: u32,
    pub amount: u64,
}

async fn sign_request_handler(
    State(_state): State<Arc<AppState>>,
    Path(vault_id): Path<String>,
    Json(req): Json<SignRequestBody>,
) -> impl IntoResponse {
    use ogbank_core::auth::{AuthTree, AuthorizationTicket};
    use ogbank_core::ceremony;
    use ogbank_core::frost::{self, Identifier};
    use ogbank_core::signer::{
        compute_sighash, AuthorizationProof, ProposedTx, SignerState, SigningRequest,
    };

    // Validate auth_secret length
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

    // 1. Reconstruct ticket + tree
    let ticket = AuthorizationTicket::new(auth_secret, req.max_amount, dest_bytes, req.expiry_block);
    let mut tree = AuthTree::new();
    let leaf_index = tree.insert(ticket.commitment()).unwrap();
    let merkle_proof = tree.proof(leaf_index).unwrap();
    let tree_root = tree.root();

    // 2. Build signing request
    let tx_data = format!("vault={vault_id},amount={}", req.amount).into_bytes();
    let sighash = compute_sighash(&tx_data);

    let auth_proof = AuthorizationProof {
        auth_secret: ticket.auth_secret,
        auth_nullifier_hash: ticket.nullifier_hash(),
        max_amount: ticket.max_amount,
        destination_hash: ticket.destination_hash,
        expiry_block: ticket.expiry_block,
        merkle_proof,
    };

    let proposed_tx = ProposedTx {
        total_spend_value: req.amount,
        recipient_address: dest_bytes.to_vec(),
        tx_data,
    };

    let signing_request = SigningRequest {
        sighash,
        auth_proof,
        proposed_tx,
    };

    // 3. Validate (7 checks)
    let signer_state = SignerState::new(tree_root, req.current_block);
    if let Err(e) = signer_state.validate(&signing_request) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "vault_id": vault_id,
                "valid": false,
                "error": format!("{e:?}")
            })),
        );
    }

    // 4. Run ceremony (MVP simulation — in production Signer is a separate daemon)
    let mut rng = rand::rngs::OsRng;
    let vault_keys = match frost::run_dkg_local(&mut rng) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("DKG failed: {e}") })),
            );
        }
    };

    let signer_ids = [
        Identifier::try_from(frost::OWNER_A_ID).unwrap(),
        Identifier::try_from(frost::RELAYER_ID).unwrap(),
    ];

    // Re-create signer state (the validate above consumed the nullifier check state)
    let mut signer_state2 = SignerState::new(tree_root, req.current_block);

    match ceremony::run_ceremony_local(
        &vault_keys,
        &signer_ids,
        &signing_request,
        &mut signer_state2,
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
                    "vault_id": vault_id,
                    "valid": true,
                    "signature": hex::encode(sig_bytes),
                    "verification": if verified { "passed" } else { "failed" }
                })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "vault_id": vault_id,
                "valid": true,
                "error": format!("ceremony failed: {e:?}")
            })),
        ),
    }
}

// --- Balance ---

async fn balance_handler(
    State(state): State<Arc<AppState>>,
    Path(position_id): Path<String>,
) -> impl IntoResponse {
    let conn = state.db.lock().unwrap();

    match db::get_position_balance(&conn, &position_id) {
        Ok((balance, borrowed, notes)) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "position_id": position_id,
                "balance_zat": balance,
                "borrowed_zat": borrowed,
                "note_count": notes
            })),
        ),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "position not found" })),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn test_state() -> Arc<AppState> {
        let conn = Connection::open_in_memory().expect("in-memory db");
        db::init_db(&conn).expect("init_db");
        Arc::new(AppState {
            db: Mutex::new(conn),
            sk_passphrase: "test-passphrase".to_string(),
        })
    }

    async fn body_json(body: Body) -> serde_json::Value {
        let bytes = body.collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn test_register_creates_position() {
        let state = test_state();
        let app = router(state.clone());

        let req = Request::builder()
            .method("POST")
            .uri("/register")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_string(&serde_json::json!({
                    "position_id": "pos-1",
                    "address": "zs1test",
                    "sk": hex::encode([1u8; 32]),
                    "fvk": hex::encode([2u8; 96]),
                    "ivk": hex::encode([3u8; 64]),
                }))
                .unwrap(),
            ))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let json = body_json(resp.into_body()).await;
        assert_eq!(json["position_id"], "pos-1");
        assert_eq!(json["status"], "registered");
    }

    #[tokio::test]
    async fn test_register_encrypts_sk() {
        let state = test_state();
        let app = router(state.clone());

        let sk_plain = [42u8; 32];
        let req = Request::builder()
            .method("POST")
            .uri("/register")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_string(&serde_json::json!({
                    "position_id": "pos-enc",
                    "address": "zs1test",
                    "sk": hex::encode(sk_plain),
                    "fvk": hex::encode([2u8; 96]),
                    "ivk": hex::encode([3u8; 64]),
                }))
                .unwrap(),
            ))
            .unwrap();

        let _resp = app.oneshot(req).await.unwrap();

        // Verify the stored sk is encrypted (not plaintext)
        let conn = state.db.lock().unwrap();
        let stored_sk: Vec<u8> = conn
            .query_row(
                "SELECT sk_encrypted FROM positions WHERE id = 'pos-enc'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_ne!(
            stored_sk,
            sk_plain.to_vec(),
            "stored sk must be encrypted, not plaintext"
        );
        assert!(
            stored_sk.len() > 32,
            "encrypted sk must be larger than raw sk (salt + nonce + ciphertext + tag)"
        );
    }

    #[tokio::test]
    async fn test_balance_returns_correct() {
        let state = test_state();

        // Insert a position directly into DB
        {
            let conn = state.db.lock().unwrap();
            db::insert_position(&conn, "pos-bal", "zs1test", &[2; 96], &[0; 64], &[3; 64])
                .unwrap();
            db::update_balance(&conn, "pos-bal", 5_000_000).unwrap();
        }

        let app = router(state);
        let req = Request::builder()
            .method("GET")
            .uri("/balance/pos-bal")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let json = body_json(resp.into_body()).await;
        assert_eq!(json["balance_zat"], 5_000_000);
        assert_eq!(json["borrowed_zat"], 0);
    }

    #[tokio::test]
    async fn test_balance_not_found() {
        let state = test_state();
        let app = router(state);

        let req = Request::builder()
            .method("GET")
            .uri("/balance/nonexistent")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_scan_placeholder() {
        let state = test_state();

        // Register a position first
        {
            let conn = state.db.lock().unwrap();
            db::insert_position(&conn, "pos-scan", "zs1test", &[2; 96], &[0; 64], &[3; 64])
                .unwrap();
        }

        let app = router(state);
        let req = Request::builder()
            .method("POST")
            .uri("/scan/pos-scan")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"txid":"abc123"}"#))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let json = body_json(resp.into_body()).await;
        assert_eq!(json["position_id"], "pos-scan");
    }

    #[tokio::test]
    async fn test_vault_create() {
        let state = test_state();
        let app = router(state.clone());

        let req = Request::builder()
            .method("POST")
            .uri("/vault/create")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_string(&serde_json::json!({
                    "vault_id": "vault-1",
                    "vault_address": "zs1vault",
                    "group_public_key": hex::encode([1u8; 32]),
                }))
                .unwrap(),
            ))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let json = body_json(resp.into_body()).await;
        assert_eq!(json["vault_id"], "vault-1");
        assert_eq!(json["status"], "registered");

        // Verify in DB
        let conn = state.db.lock().unwrap();
        let (addr, gpk, status) = db::get_vault(&conn, "vault-1").unwrap();
        assert_eq!(addr, "zs1vault");
        assert_eq!(gpk, vec![1u8; 32]);
        assert_eq!(status, "active");
    }

    #[tokio::test]
    async fn test_sign_request_valid() {
        let state = test_state();
        let app = router(state);

        let auth_secret = [42u8; 32];
        let req = Request::builder()
            .method("POST")
            .uri("/vault/test-vault/sign-request")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_string(&serde_json::json!({
                    "auth_secret": hex::encode(auth_secret),
                    "max_amount": 1_000_000u64,
                    "destination": "recipient-address",
                    "expiry_block": 500_000u32,
                    "current_block": 100u32,
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
        assert!(json["signature"].as_str().unwrap().len() > 0, "signature must be non-empty");
    }

    #[tokio::test]
    async fn test_sign_request_expired() {
        let state = test_state();
        let app = router(state);

        let auth_secret = [42u8; 32];
        let req = Request::builder()
            .method("POST")
            .uri("/vault/test-vault/sign-request")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_string(&serde_json::json!({
                    "auth_secret": hex::encode(auth_secret),
                    "max_amount": 1_000_000u64,
                    "destination": "recipient-address",
                    "expiry_block": 100u32,
                    "current_block": 200u32,
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
    async fn test_sign_request_amount_exceeded() {
        let state = test_state();
        let app = router(state);

        let auth_secret = [42u8; 32];
        let req = Request::builder()
            .method("POST")
            .uri("/vault/test-vault/sign-request")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_string(&serde_json::json!({
                    "auth_secret": hex::encode(auth_secret),
                    "max_amount": 100_000u64,
                    "destination": "recipient-address",
                    "expiry_block": 500_000u32,
                    "current_block": 100u32,
                    "amount": 999_999u64,
                }))
                .unwrap(),
            ))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let json = body_json(resp.into_body()).await;
        assert_eq!(json["valid"], false);
        assert!(json["error"].as_str().unwrap().contains("AmountExceeded"));
    }
}

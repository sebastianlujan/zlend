// Phase 6: REST API Endpoints
//
// POST /register — store viewing/spending keys for a new position
// POST /scan/:id — trial decrypt a Zcash tx to find deposits (uses ivk)
// GET  /balance/:id — return current collateral balance
//
// Reference: docs/technical/08_mvp.md

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
}

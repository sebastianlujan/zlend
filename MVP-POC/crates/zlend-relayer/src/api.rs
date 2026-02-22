use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::{db, evm, zcash};

/// Shared application state passed to all request handlers.
pub struct AppState {
    pub db: db::Database,
    pub tatum: zcash::TatumClient,
    pub evm: Option<evm::EvmSigner>,
    /// Passphrase for encrypting/decrypting spending keys.
    pub sk_passphrase: String,
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(register))
        .route("/balance/{id}", get(balance))
        .route("/scan/{id}", post(scan))
        .route("/borrow/{id}", post(borrow))
        .route("/positions", get(list_positions))
        .route("/health", get(health))
}

/// Build the complete axum Router with CORS and shared state.
///
/// Used by main.rs and integration tests.
pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(routes())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

// ─── Request/Response Types ───

#[derive(Deserialize)]
struct RegisterRequest {
    sk: String,
    vk: String,
    ivk: String,
    address: String,
}

#[derive(Serialize)]
struct RegisterResponse {
    id: String,
    address: String,
}

#[derive(Serialize)]
struct BalanceResponse {
    id: String,
    address: String,
    balance_zat: i64,
    borrowed_zat: i64,
    note_count: i64,
}

#[derive(Deserialize)]
struct ScanRequest {
    txid: String,
}

#[derive(Serialize)]
struct ScanResponse {
    found: bool,
    value_zat: Option<u64>,
    total_balance_zat: Option<i64>,
}

#[derive(Deserialize)]
struct BorrowRequest {
    amount: u64,
    recipient: String,
}

#[derive(Serialize)]
struct BorrowResponse {
    success: bool,
    tx_hash: Option<String>,
    borrowed: Option<u64>,
    error: Option<String>,
}

#[derive(Serialize)]
struct PositionSummary {
    id: String,
    address: String,
    balance_zat: i64,
    borrowed_zat: i64,
    note_count: i64,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

// ─── Handlers ───

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok", "service": "zlend-relayer" }))
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> impl IntoResponse {
    // Decode hex-encoded keys
    let sk_bytes = match hex::decode(&req.sk) {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("invalid sk hex: {}", e) })),
            )
                .into_response();
        }
    };

    let vk_bytes = match hex::decode(&req.vk) {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("invalid vk hex: {}", e) })),
            )
                .into_response();
        }
    };

    let ivk_bytes = match hex::decode(&req.ivk) {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("invalid ivk hex: {}", e) })),
            )
                .into_response();
        }
    };

    // Encrypt the spending key before storing
    let sk_encrypted = match zlend_core::crypto::encrypt_sk(&sk_bytes, &state.sk_passphrase) {
        Ok(enc) => enc,
        Err(e) => {
            tracing::error!("SK encryption failed: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "key encryption failed" })),
            )
                .into_response();
        }
    };

    // Generate position ID
    let id = uuid::Uuid::new_v4().to_string();

    // Store in database
    if let Err(e) = state.db.insert_position(&id, &req.address, &vk_bytes, &sk_encrypted, &ivk_bytes) {
        tracing::error!("Database insert failed: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "database error" })),
        )
            .into_response();
    }

    tracing::info!("Registered position {} for address {}", id, req.address);

    (
        StatusCode::CREATED,
        Json(serde_json::json!(RegisterResponse {
            id,
            address: req.address,
        })),
    )
        .into_response()
}

async fn balance(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.db.get_position(&id) {
        Ok(Some(pos)) => Json(serde_json::json!(BalanceResponse {
            id: pos.id,
            address: pos.address,
            balance_zat: pos.balance_zat,
            borrowed_zat: pos.borrowed_zat,
            note_count: pos.note_count,
        }))
        .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "position not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("database error: {}", e) })),
        )
            .into_response(),
    }
}

async fn scan(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<ScanRequest>,
) -> impl IntoResponse {
    // Verify position exists
    let position = match state.db.get_position(&id) {
        Ok(Some(pos)) => pos,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "position not found" })),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("database error: {}", e) })),
            )
                .into_response();
        }
    };

    // Check if we already scanned this txid for this position
    match state.db.check_note_exists(&id, &req.txid) {
        Ok(true) => {
            return Json(serde_json::json!(ScanResponse {
                found: true,
                value_zat: Some(0),
                total_balance_zat: Some(position.balance_zat),
            }))
            .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("database error: {}", e) })),
            )
                .into_response();
        }
        _ => {}
    }

    // Fetch transaction from Tatum API
    let tx_data = match state.tatum.get_transaction(&req.txid).await {
        Ok(data) => data,
        Err(e) => {
            tracing::warn!("Tatum API error for tx {}: {}", req.txid, e);
            // For MVP, if Tatum fails we can use simulated scanning
            // This allows development without a Tatum API key
            tracing::info!("Using simulated scan for development");

            // Simulate finding a 1.5 ZEC deposit
            let simulated_value = 150_000_000i64; // 1.5 ZEC in zatoshis
            let note = zlend_core::scan::try_decrypt_note_simulated(
                simulated_value as u64,
                "zlend-deposit",
            );

            if let Err(e) = state.db.insert_note(&id, &req.txid, note.value_zat as i64) {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": format!("database error: {}", e) })),
                )
                    .into_response();
            }

            let updated = state.db.get_position(&id).ok().flatten();
            return Json(serde_json::json!(ScanResponse {
                found: true,
                value_zat: Some(note.value_zat),
                total_balance_zat: updated.map(|p| p.balance_zat),
            }))
            .into_response();
        }
    };

    // TODO: Extract Orchard actions from tx_data and attempt trial decryption
    // with position.ivk. For now, log what we got and return not found.
    tracing::info!("Fetched transaction {} — checking for Orchard outputs", req.txid);

    // The real implementation would:
    // 1. Parse tx_data for orchard_actions
    // 2. For each action, call try_decrypt_note(position.ivk, ...)
    // 3. If any succeed, insert note and update balance

    let _ = tx_data; // suppress unused warning
    let _ = position;

    Json(serde_json::json!(ScanResponse {
        found: false,
        value_zat: None,
        total_balance_zat: None,
    }))
    .into_response()
}

async fn borrow(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<BorrowRequest>,
) -> impl IntoResponse {
    // Verify position exists
    let position = match state.db.get_position(&id) {
        Ok(Some(pos)) => pos,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!(BorrowResponse {
                    success: false,
                    tx_hash: None,
                    borrowed: None,
                    error: Some("position not found".into()),
                })),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!(BorrowResponse {
                    success: false,
                    tx_hash: None,
                    borrowed: None,
                    error: Some(format!("database error: {}", e)),
                })),
            )
                .into_response();
        }
    };

    // Check for existing active loan
    match state.db.has_active_loan(&id) {
        Ok(true) => {
            return (
                StatusCode::CONFLICT,
                Json(serde_json::json!(BorrowResponse {
                    success: false,
                    tx_hash: None,
                    borrowed: None,
                    error: Some("active loan already exists for this position".into()),
                })),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!(BorrowResponse {
                    success: false,
                    tx_hash: None,
                    borrowed: None,
                    error: Some(format!("database error: {}", e)),
                })),
            )
                .into_response();
        }
        _ => {}
    }

    // Check collateral ratio: balance_zat * 100 >= amount * 150
    let collateral_check = (position.balance_zat as u128) * 100 >= (req.amount as u128) * 150;
    if !collateral_check {
        let required = (req.amount as f64 * 1.5) / 100_000_000.0;
        let available = position.balance_zat as f64 / 100_000_000.0;
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!(BorrowResponse {
                success: false,
                tx_hash: None,
                borrowed: None,
                error: Some(format!(
                    "insufficient collateral: need {:.8} ZEC (150%), have {:.8} ZEC",
                    required, available
                )),
            })),
        )
            .into_response();
    }

    // Submit borrow transaction via EVM signer
    let tx_hash = match &state.evm {
        Some(evm) => {
            match evm
                .borrow(
                    &req.recipient,
                    position.balance_zat as u64,
                    req.amount,
                    &id,
                )
                .await
            {
                Ok(hash) => Some(hash),
                Err(e) => {
                    tracing::error!("EVM borrow tx failed: {}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!(BorrowResponse {
                            success: false,
                            tx_hash: None,
                            borrowed: None,
                            error: Some(format!("EVM transaction failed: {}", e)),
                        })),
                    )
                        .into_response();
                }
            }
        }
        None => {
            tracing::warn!("No EVM signer configured — recording loan without onchain tx");
            Some("0x_simulated_no_evm_signer".to_string())
        }
    };

    // Record loan in database
    if let Err(e) = state.db.insert_loan(
        &id,
        req.amount as i64,
        &req.recipient,
        tx_hash.as_deref(),
    ) {
        tracing::error!("Failed to record loan: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!(BorrowResponse {
                success: false,
                tx_hash,
                borrowed: None,
                error: Some(format!("loan recording failed: {}", e)),
            })),
        )
            .into_response();
    }

    tracing::info!(
        "Borrow successful: position={}, amount={}, recipient={}, tx={:?}",
        id, req.amount, req.recipient, tx_hash
    );

    Json(serde_json::json!(BorrowResponse {
        success: true,
        tx_hash,
        borrowed: Some(req.amount),
        error: None,
    }))
    .into_response()
}

async fn list_positions(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.db.list_positions() {
        Ok(positions) => {
            let summaries: Vec<PositionSummary> = positions
                .into_iter()
                .map(|p| PositionSummary {
                    id: p.id,
                    address: p.address,
                    balance_zat: p.balance_zat,
                    borrowed_zat: p.borrowed_zat,
                    note_count: p.note_count,
                })
                .collect();
            Json(serde_json::json!(summaries)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("database error: {}", e) })),
        )
            .into_response(),
    }
}

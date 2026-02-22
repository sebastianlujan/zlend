//! End-to-end tests for the ZLend relayer REST API.
//!
//! Tests the full lending lifecycle through the HTTP API:
//!   1. Health check
//!   2. Register a position
//!   3. Scan a transaction (simulated — no Tatum API key)
//!   4. Check balance
//!   5. Borrow against collateral
//!   6. Verify final balance
//!   7. Collateral ratio enforcement
//!   8. Double-borrow prevention

use axum::body::Body;
use axum::http::{Request, StatusCode};
use std::sync::Arc;
use tower::util::ServiceExt;

use zlend_relayer::{
    api::{build_router, AppState},
    db::Database,
    zcash::TatumClient,
};

/// Create a test app with in-memory database and no external services.
fn test_app() -> axum::Router {
    let database = Database::new(":memory:").unwrap();
    database.init().unwrap();

    let state = Arc::new(AppState {
        db: database,
        tatum: TatumClient::new(""), // empty key → simulated scan fallback
        evm: None,                   // no EVM → simulated borrow
        sk_passphrase: "test-passphrase".to_string(),
    });

    build_router(state)
}

/// Helper: read response body as JSON
async fn body_json(response: axum::http::Response<Body>) -> serde_json::Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

/// Generate test keys for registration
fn test_keys() -> (String, String, String, String) {
    let network = zlend_core::keys::testnet();
    let keys = zlend_core::keys::generate_keys(&network).unwrap();
    (keys.sk.clone(), keys.fvk.clone(), keys.ivk.clone(), keys.address.clone())
}

// ─── Tests ───

#[tokio::test]
async fn test_health_endpoint() {
    let app = test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["status"], "ok");
    assert_eq!(json["service"], "zlend-relayer");
}

#[tokio::test]
async fn test_full_lending_lifecycle() {
    let app = test_app();
    let (sk, vk, ivk, address) = test_keys();

    // ─── Step 1: Register ───
    let register_body = serde_json::json!({
        "sk": sk,
        "vk": vk,
        "ivk": ivk,
        "address": address,
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/register")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&register_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let json = body_json(response).await;
    let position_id = json["id"].as_str().unwrap().to_string();
    assert!(!position_id.is_empty(), "Position ID must be returned");
    assert_eq!(json["address"], address);

    // ─── Step 2: Check initial balance (should be 0) ───
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/balance/{}", position_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["balance_zat"], 0);
    assert_eq!(json["borrowed_zat"], 0);
    assert_eq!(json["note_count"], 0);

    // ─── Step 3: Scan a transaction (simulated — injects 1.5 ZEC) ───
    let scan_body = serde_json::json!({
        "txid": "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/scan/{}", position_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&scan_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["found"], true, "Simulated scan must find a deposit");
    assert_eq!(json["value_zat"], 150_000_000, "Simulated deposit is 1.5 ZEC");
    assert_eq!(
        json["total_balance_zat"], 150_000_000,
        "Total balance after deposit"
    );

    // ─── Step 4: Check balance after deposit ───
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/balance/{}", position_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["balance_zat"], 150_000_000, "Balance must be 1.5 ZEC");
    assert_eq!(json["note_count"], 1, "Must have 1 note");

    // ─── Step 5: Borrow against collateral ───
    // With 150M zat collateral and 150% ratio, max borrow = 100M zat
    let borrow_body = serde_json::json!({
        "amount": 100_000_000_u64,
        "recipient": "0x1234567890abcdef1234567890abcdef12345678",
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/borrow/{}", position_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&borrow_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["success"], true, "Borrow must succeed");
    assert_eq!(json["borrowed"], 100_000_000);
    assert!(json["tx_hash"].is_string(), "Must return a tx hash");

    // ─── Step 6: Check final balance ───
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/balance/{}", position_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["balance_zat"], 150_000_000, "Collateral unchanged");
    assert_eq!(json["borrowed_zat"], 100_000_000, "Borrowed 100M zat");

    // ─── Step 7: List positions ───
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/positions")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    let positions = json.as_array().unwrap();
    assert_eq!(positions.len(), 1, "Must have 1 position");
    assert_eq!(positions[0]["id"], position_id);
}

#[tokio::test]
async fn test_collateral_ratio_enforcement() {
    let app = test_app();
    let (sk, vk, ivk, address) = test_keys();

    // Register
    let register_body = serde_json::json!({
        "sk": sk, "vk": vk, "ivk": ivk, "address": address,
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/register")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&register_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let json = body_json(response).await;
    let position_id = json["id"].as_str().unwrap().to_string();

    // Scan to add 1.5 ZEC collateral
    let scan_body = serde_json::json!({ "txid": "tx_for_collateral_test" });
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/scan/{}", position_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&scan_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Try to borrow MORE than 150% collateral allows
    // 150M zat collateral / 1.5 = 100M max borrow. Request 101M.
    let borrow_body = serde_json::json!({
        "amount": 101_000_000_u64,
        "recipient": "0xdeadbeef",
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/borrow/{}", position_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&borrow_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Insufficient collateral must be rejected"
    );
    let json = body_json(response).await;
    assert_eq!(json["success"], false);
    assert!(
        json["error"]
            .as_str()
            .unwrap()
            .contains("insufficient collateral"),
        "Error must mention insufficient collateral"
    );
}

#[tokio::test]
async fn test_double_borrow_prevention() {
    let app = test_app();
    let (sk, vk, ivk, address) = test_keys();

    // Register + scan
    let register_body = serde_json::json!({
        "sk": sk, "vk": vk, "ivk": ivk, "address": address,
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/register")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&register_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let json = body_json(response).await;
    let position_id = json["id"].as_str().unwrap().to_string();

    let scan_body = serde_json::json!({ "txid": "tx_for_double_borrow_test" });
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/scan/{}", position_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&scan_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // First borrow — should succeed
    let borrow_body = serde_json::json!({
        "amount": 50_000_000_u64,
        "recipient": "0xAlice",
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/borrow/{}", position_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&borrow_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["success"], true, "First borrow must succeed");

    // Second borrow — should fail (active loan exists)
    let borrow_body2 = serde_json::json!({
        "amount": 10_000_000_u64,
        "recipient": "0xBob",
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/borrow/{}", position_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&borrow_body2).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::CONFLICT,
        "Double borrow must be rejected"
    );
    let json = body_json(response).await;
    assert_eq!(json["success"], false);
    assert!(
        json["error"]
            .as_str()
            .unwrap()
            .contains("active loan already exists"),
        "Error must mention active loan"
    );
}

#[tokio::test]
async fn test_position_not_found() {
    let app = test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/balance/nonexistent-uuid")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = body_json(response).await;
    assert!(json["error"].as_str().unwrap().contains("not found"));
}

#[tokio::test]
async fn test_duplicate_scan_idempotent() {
    let app = test_app();
    let (sk, vk, ivk, address) = test_keys();

    // Register
    let register_body = serde_json::json!({
        "sk": sk, "vk": vk, "ivk": ivk, "address": address,
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/register")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&register_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let json = body_json(response).await;
    let position_id = json["id"].as_str().unwrap().to_string();

    let scan_body = serde_json::json!({ "txid": "same-txid-scanned-twice" });

    // First scan — adds note
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/scan/{}", position_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&scan_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Second scan with same txid — should be idempotent
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/scan/{}", position_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&scan_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["found"], true);

    // Balance should still be 1.5 ZEC (not doubled)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/balance/{}", position_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let json = body_json(response).await;
    assert_eq!(
        json["balance_zat"], 150_000_000,
        "Balance must not double on duplicate scan"
    );
    assert_eq!(json["note_count"], 1, "Note count must not increase");
}

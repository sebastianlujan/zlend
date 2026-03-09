// Phase 6: OGBank Relayer
//
// axum REST API server with SQLite persistence.
//
// Reference: docs/technical/08_mvp.md

use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use rusqlite::Connection;
use tracing::info;

use ogbank_relayer::api::AppState;
use ogbank_relayer::{api, db};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ogbank_relayer=info".into()),
        )
        .init();

    let sk_passphrase = std::env::var("RELAYER_SK_PASSPHRASE")
        .unwrap_or_else(|_| "dev-passphrase-DO-NOT-USE-IN-PROD".to_string());

    let db_path = std::env::var("RELAYER_DB_PATH").unwrap_or_else(|_| "ogbank.db".to_string());

    let conn = Connection::open(&db_path)
        .with_context(|| format!("failed to open database at {db_path}"))?;
    db::init_db(&conn).context("failed to initialize database")?;

    let signer_url = std::env::var("SIGNER_URL").ok();
    let signer_client = signer_url
        .as_ref()
        .map(|url| ogbank_relayer::signer_client::SignerClient::new(url));

    let state = Arc::new(AppState {
        db: Mutex::new(conn),
        sk_passphrase,
        signer_url,
        signer_client,
        key_store: Arc::new(ogbank_core::frost::InMemoryKeyStore::new()),
    });

    let app = api::router(state);

    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .with_context(|| format!("failed to bind to {bind_addr}"))?;

    info!("ogbank-relayer listening on {bind_addr}");
    axum::serve(listener, app)
        .await
        .context("server error")?;

    Ok(())
}

use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use zlend_relayer::{api::AppState, db, evm, zcash};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "zlend_relayer=info,tower_http=info".into()),
        ))
        .init();

    // Load configuration from environment
    let sk_passphrase = std::env::var("RELAYER_SK_PASSPHRASE")
        .unwrap_or_else(|_| {
            tracing::warn!("RELAYER_SK_PASSPHRASE not set — using default (INSECURE, dev only)");
            "zlend-dev-passphrase-change-me".to_string()
        });

    let tatum_api_key = std::env::var("TATUM_API_KEY")
        .unwrap_or_else(|_| {
            tracing::warn!("TATUM_API_KEY not set — Zcash scanning will fail");
            String::new()
        });

    // Initialize EVM signer if configured
    let evm_signer = match (
        std::env::var("RELAYER_EVM_KEY"),
        std::env::var("AVALANCHE_FUJI_RPC"),
        std::env::var("ZLEND_CONTRACT_ADDRESS"),
    ) {
        (Ok(key), Ok(rpc), Ok(contract)) => {
            match evm::EvmSigner::new(&key, &rpc, &contract).await {
                Ok(signer) => {
                    tracing::info!("EVM signer initialized for contract {}", contract);
                    Some(signer)
                }
                Err(e) => {
                    tracing::warn!("EVM signer initialization failed: {} — borrow endpoint will return errors", e);
                    None
                }
            }
        }
        _ => {
            tracing::warn!("EVM config incomplete (need RELAYER_EVM_KEY, AVALANCHE_FUJI_RPC, ZLEND_CONTRACT_ADDRESS) — borrow endpoint disabled");
            None
        }
    };

    // Initialize database
    let database = db::Database::new("zlend.db")?;
    database.init()?;
    tracing::info!("SQLite database initialized");

    // Build shared state
    let state = Arc::new(AppState {
        db: database,
        tatum: zcash::TatumClient::new(&tatum_api_key),
        evm: evm_signer,
        sk_passphrase,
    });

    // Build router and start server
    let app = zlend_relayer::api::build_router(state);

    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".into());
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("ZLend relayer listening on {}", bind_addr);

    axum::serve(listener, app).await?;

    Ok(())
}

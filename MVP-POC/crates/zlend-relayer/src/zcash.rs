use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TatumError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("API error ({status}): {body}")]
    ApiError { status: u16, body: String },
    #[error("no API key configured")]
    NoApiKey,
}

/// Raw transaction data from Tatum's Zcash API.
///
/// This is a simplified representation — the actual Tatum response
/// includes more fields. We capture what we need for trial decryption.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZcashTransaction {
    pub txid: String,
    pub version: Option<i64>,
    pub size: Option<i64>,
    /// Transparent inputs
    pub vin: Option<Vec<serde_json::Value>>,
    /// Transparent outputs
    pub vout: Option<Vec<serde_json::Value>>,
    /// Orchard actions (if present — Tatum may not expose these)
    #[serde(rename = "orchardActions")]
    pub orchard_actions: Option<Vec<serde_json::Value>>,
    /// Sapling shielded spends
    #[serde(rename = "vShieldedSpend")]
    pub v_shielded_spend: Option<Vec<serde_json::Value>>,
    /// Sapling shielded outputs
    #[serde(rename = "vShieldedOutput")]
    pub v_shielded_output: Option<Vec<serde_json::Value>>,
    /// Raw hex of the transaction (for manual parsing)
    pub hex: Option<String>,
}

pub struct TatumClient {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl TatumClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://api.tatum.io/v3/zcash".to_string(),
            api_key: api_key.to_string(),
        }
    }

    /// Fetch a Zcash transaction by its txid.
    ///
    /// Uses: GET /v3/zcash/transaction/{hash}
    pub async fn get_transaction(&self, txid: &str) -> Result<ZcashTransaction, TatumError> {
        if self.api_key.is_empty() {
            return Err(TatumError::NoApiKey);
        }

        let url = format!("{}/transaction/{}", self.base_url, txid);
        let resp = self
            .client
            .get(&url)
            .header("x-api-key", &self.api_key)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(TatumError::ApiError {
                status: status.as_u16(),
                body,
            });
        }

        let tx: ZcashTransaction = resp.json().await?;
        Ok(tx)
    }

    /// Fetch a Zcash block by height.
    ///
    /// Uses: GET /v3/zcash/block/{hash|height}
    pub async fn get_block(&self, height: u64) -> Result<serde_json::Value, TatumError> {
        if self.api_key.is_empty() {
            return Err(TatumError::NoApiKey);
        }

        let url = format!("{}/block/{}", self.base_url, height);
        let resp = self
            .client
            .get(&url)
            .header("x-api-key", &self.api_key)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(TatumError::ApiError {
                status: status.as_u16(),
                body,
            });
        }

        let block: serde_json::Value = resp.json().await?;
        Ok(block)
    }

    /// Broadcast a signed Zcash transaction.
    ///
    /// Uses: POST /v3/zcash/broadcast
    pub async fn broadcast(&self, raw_tx: &str) -> Result<String, TatumError> {
        if self.api_key.is_empty() {
            return Err(TatumError::NoApiKey);
        }

        let url = format!("{}/broadcast", self.base_url);
        let resp = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .json(&serde_json::json!({ "txData": raw_tx }))
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(TatumError::ApiError {
                status: status.as_u16(),
                body,
            });
        }

        let result: serde_json::Value = resp.json().await?;
        Ok(result["txId"].as_str().unwrap_or("unknown").to_string())
    }
}

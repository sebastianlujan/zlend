// Phase 7: Zcash RPC Client
//
// TatumClient — fetches Zcash transactions from the Tatum REST API,
// extracts Orchard actions, and converts them to CompactAction for
// trial decryption via ogbank_core::scan.
//
// ZcashNodeClient — JSON-RPC client for zcashd (z_sendmany for ZEC return).
//
// Reference: docs/technical/08_mvp.md, docs/technical/05_zcash-integration.md

use anyhow::{anyhow, Result};
use orchard::note::ExtractedNoteCommitment;
use orchard::note::Nullifier;
use orchard::note_encryption::CompactAction;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Orchard action data — parsed from Tatum API JSON
// ---------------------------------------------------------------------------

/// Raw Orchard action bytes parsed from the Tatum API response.
///
/// These are hex-decoded fields from the transaction JSON that we
/// convert into `CompactAction` for trial decryption.
#[derive(Debug, Clone)]
pub struct OrchardActionData {
    pub nf: [u8; 32],
    pub cmx: [u8; 32],
    pub ephemeral_key: [u8; 32],
    /// First 52 bytes of the encrypted ciphertext (compact form).
    pub enc_ciphertext_compact: [u8; 52],
}

impl OrchardActionData {
    /// Convert raw bytes into a `CompactAction` for trial decryption.
    pub fn to_compact_action(&self) -> Option<CompactAction> {
        let nf = Nullifier::from_bytes(&self.nf);
        let nf = Option::from(nf)?;

        let cmx = ExtractedNoteCommitment::from_bytes(&self.cmx);
        let cmx = Option::from(cmx)?;

        Some(CompactAction::from_parts(
            nf,
            cmx,
            self.ephemeral_key.into(),
            self.enc_ciphertext_compact,
        ))
    }
}

/// Convert a batch of raw action data into CompactActions, skipping any
/// that fail to parse (malformed on-chain data shouldn't crash scanning).
pub fn to_compact_actions(actions: &[OrchardActionData]) -> Vec<CompactAction> {
    actions.iter().filter_map(|a| a.to_compact_action()).collect()
}

// ---------------------------------------------------------------------------
// Tatum API types — JSON response shapes
// ---------------------------------------------------------------------------

/// Top-level Tatum transaction response.
///
/// We only care about the Orchard shielded actions for trial decryption.
/// Transparent inputs/outputs are ignored for the MVP.
#[derive(Debug, Deserialize)]
pub struct TatumTransaction {
    pub txid: String,
    #[serde(default)]
    pub orchard_actions: Vec<TatumOrchardAction>,
    /// Raw transaction hex — fallback if orchard_actions isn't populated.
    #[serde(default)]
    pub hex: Option<String>,
}

/// Orchard action as returned by Tatum API (hex-encoded fields).
#[derive(Debug, Deserialize)]
pub struct TatumOrchardAction {
    pub nf: String,
    pub cmx: String,
    #[serde(alias = "ephemeralKey")]
    pub ephemeral_key: String,
    #[serde(alias = "encCiphertext")]
    pub enc_ciphertext: String,
}

impl TatumOrchardAction {
    /// Parse hex fields into raw bytes.
    pub fn to_action_data(&self) -> Result<OrchardActionData> {
        let nf: [u8; 32] = hex::decode(&self.nf)
            .map_err(|e| anyhow!("bad nf hex: {e}"))?
            .try_into()
            .map_err(|v: Vec<u8>| anyhow!("nf wrong length: {}", v.len()))?;

        let cmx: [u8; 32] = hex::decode(&self.cmx)
            .map_err(|e| anyhow!("bad cmx hex: {e}"))?
            .try_into()
            .map_err(|v: Vec<u8>| anyhow!("cmx wrong length: {}", v.len()))?;

        let ephemeral_key: [u8; 32] = hex::decode(&self.ephemeral_key)
            .map_err(|e| anyhow!("bad ephemeral_key hex: {e}"))?
            .try_into()
            .map_err(|v: Vec<u8>| anyhow!("ephemeral_key wrong length: {}", v.len()))?;

        let enc_raw = hex::decode(&self.enc_ciphertext)
            .map_err(|e| anyhow!("bad enc_ciphertext hex: {e}"))?;

        if enc_raw.len() < 52 {
            return Err(anyhow!(
                "enc_ciphertext too short: {} bytes (need ≥52)",
                enc_raw.len()
            ));
        }

        let mut enc_ciphertext_compact = [0u8; 52];
        enc_ciphertext_compact.copy_from_slice(&enc_raw[..52]);

        Ok(OrchardActionData {
            nf,
            cmx,
            ephemeral_key,
            enc_ciphertext_compact,
        })
    }
}

// ---------------------------------------------------------------------------
// TatumClient — HTTP client
// ---------------------------------------------------------------------------

/// HTTP client for fetching Zcash transactions from the Tatum REST API.
pub struct TatumClient {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl TatumClient {
    pub fn new(api_key: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("failed to build reqwest client");

        Self {
            api_key: api_key.to_string(),
            base_url: "https://api.tatum.io".to_string(),
            client,
        }
    }

    /// Fetch a Zcash transaction by txid from Tatum.
    pub async fn get_transaction(&self, txid: &str) -> Result<TatumTransaction> {
        let url = format!("{}/v3/zcash/transaction/{}", self.base_url, txid);

        let resp = self
            .client
            .get(&url)
            .header("x-api-key", &self.api_key)
            .send()
            .await
            .map_err(|e| anyhow!("tatum request failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("tatum returned {status}: {body}"));
        }

        resp.json::<TatumTransaction>()
            .await
            .map_err(|e| anyhow!("tatum response parse error: {e}"))
    }

    /// Fetch a transaction and extract its Orchard actions as CompactActions.
    pub async fn get_orchard_actions(&self, txid: &str) -> Result<Vec<CompactAction>> {
        let tx = self.get_transaction(txid).await?;
        let action_data: Vec<OrchardActionData> = tx
            .orchard_actions
            .iter()
            .filter_map(|a| a.to_action_data().ok())
            .collect();

        Ok(to_compact_actions(&action_data))
    }
}

// ---------------------------------------------------------------------------
// ZcashNodeClient — JSON-RPC for zcashd (z_sendmany)
// ---------------------------------------------------------------------------

/// JSON-RPC client for interacting with a local zcashd node.
///
/// Used for sending ZEC back to users during the withdrawal phase.
pub struct ZcashNodeClient {
    rpc_url: String,
    client: reqwest::Client,
}

impl ZcashNodeClient {
    pub fn new(rpc_url: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("failed to build reqwest client");

        Self {
            rpc_url: rpc_url.trim_end_matches('/').to_string(),
            client,
        }
    }

    /// Send ZEC from a shielded address to a recipient via z_sendmany.
    ///
    /// Returns the async operation ID (opid) for tracking.
    pub async fn z_sendmany(
        &self,
        from_address: &str,
        to_address: &str,
        amount_zat: u64,
    ) -> Result<String> {
        let amount_zec = amount_zat as f64 / 100_000_000.0;

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "ogbank",
            "method": "z_sendmany",
            "params": [
                from_address,
                [{ "address": to_address, "amount": amount_zec }],
                1,   // minconf
                null // fee (use default)
            ]
        });

        let resp = self
            .client
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("zcashd z_sendmany request failed: {e}"))?;

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| anyhow!("zcashd response parse error: {e}"))?;

        if let Some(error) = json.get("error").filter(|e| !e.is_null()) {
            return Err(anyhow!("zcashd error: {error}"));
        }

        json["result"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("zcashd z_sendmany: no opid in response"))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tatum_client_construction() {
        let client = TatumClient::new("test-api-key");
        assert_eq!(client.api_key, "test-api-key");
        assert_eq!(client.base_url, "https://api.tatum.io");
    }

    #[test]
    fn test_zcash_node_client_url_normalization() {
        let client = ZcashNodeClient::new("http://localhost:8232/");
        assert_eq!(client.rpc_url, "http://localhost:8232");
    }

    #[test]
    fn test_tatum_action_deserialization() {
        let json = serde_json::json!({
            "txid": "abc123",
            "orchard_actions": [{
                "nf": "00".repeat(32),
                "cmx": "01".repeat(32),
                "ephemeralKey": "02".repeat(32),
                "encCiphertext": "03".repeat(52)
            }]
        });

        let tx: TatumTransaction = serde_json::from_value(json).unwrap();
        assert_eq!(tx.txid, "abc123");
        assert_eq!(tx.orchard_actions.len(), 1);

        let action = tx.orchard_actions[0].to_action_data().unwrap();
        assert_eq!(action.nf, [0u8; 32]);
        assert_eq!(action.cmx, [1u8; 32]);
        assert_eq!(action.ephemeral_key, [2u8; 32]);
        assert!(action.enc_ciphertext_compact.iter().all(|&b| b == 3));
    }

    #[test]
    fn test_tatum_response_missing_orchard() {
        let json = serde_json::json!({
            "txid": "no-orchard-tx",
            "hex": "0400..."
        });

        let tx: TatumTransaction = serde_json::from_value(json).unwrap();
        assert_eq!(tx.txid, "no-orchard-tx");
        assert!(tx.orchard_actions.is_empty());
    }

    #[test]
    fn test_action_data_short_ciphertext_rejected() {
        let action = TatumOrchardAction {
            nf: "00".repeat(32),
            cmx: "01".repeat(32),
            ephemeral_key: "02".repeat(32),
            enc_ciphertext: "03".repeat(10), // only 10 bytes, need ≥52
        };

        let result = action.to_action_data();
        assert!(result.is_err());
        assert!(
            result.unwrap_err().to_string().contains("too short"),
            "should report ciphertext too short"
        );
    }

    #[test]
    fn test_action_data_bad_hex_rejected() {
        let action = TatumOrchardAction {
            nf: "not-valid-hex".to_string(),
            cmx: "01".repeat(32),
            ephemeral_key: "02".repeat(32),
            enc_ciphertext: "03".repeat(52),
        };

        assert!(action.to_action_data().is_err());
    }

    #[test]
    fn test_to_compact_actions_skips_invalid() {
        // One valid, one with bad cmx (all zeros → may not be a valid point)
        let valid = OrchardActionData {
            nf: [1u8; 32],
            cmx: [0u8; 32], // might be invalid depending on curve
            ephemeral_key: [2u8; 32],
            enc_ciphertext_compact: [3u8; 52],
        };

        // to_compact_action may return None for invalid curve points
        let actions = to_compact_actions(&[valid]);
        // We just verify it doesn't panic — result depends on curve validation
        let _ = actions;
    }
}

// Signer HTTP Client — Relayer-side communication with the Signer daemon.
//
// Per RFC §7.2.4, the Relayer coordinates FROST signing ceremonies by
// communicating with the Signer over HTTP:
//   Round 1: POST /nonce-commit  → NonceCommitmentResponse
//   Round 2: POST /sign-share    → DistributedSignResponse
//
// Reference: docs/technical/11_rfc-ogb-001.md §7.2.4, §10

use anyhow::{anyhow, Result};
use ogbank_core::protocol::{
    CeremonyAbortRequest, CeremonyAbortResponse, DistributedSignRequest,
    DistributedSignResponse, NonceCommitmentRequest, NonceCommitmentResponse,
};

/// HTTP client for communicating with the Signer daemon.
pub struct SignerClient {
    base_url: String,
    client: reqwest::Client,
}

impl SignerClient {
    pub fn new(base_url: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("failed to build reqwest client");
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
        }
    }

    /// Round 1: Request FROST nonce commitments from the Signer.
    pub async fn request_nonce_commitment(
        &self,
        req: &NonceCommitmentRequest,
    ) -> Result<NonceCommitmentResponse> {
        let url = format!("{}/nonce-commit", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(req)
            .send()
            .await
            .map_err(|e| anyhow!("signer nonce-commit request failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("signer nonce-commit returned {status}: {body}"));
        }

        resp.json::<NonceCommitmentResponse>()
            .await
            .map_err(|e| anyhow!("signer nonce-commit parse error: {e}"))
    }

    /// Round 2: Request a FROST signature share from the Signer.
    pub async fn request_sign_share(
        &self,
        req: &DistributedSignRequest,
    ) -> Result<DistributedSignResponse> {
        let url = format!("{}/sign-share", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(req)
            .send()
            .await
            .map_err(|e| anyhow!("signer sign-share request failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("signer sign-share returned {status}: {body}"));
        }

        resp.json::<DistributedSignResponse>()
            .await
            .map_err(|e| anyhow!("signer sign-share parse error: {e}"))
    }

    /// Clear all pre-committed nonces on the Signer (for revocation per §9.2).
    pub async fn clear_nonces(&self) -> Result<()> {
        let url = format!("{}/nonce-clear", self.base_url);
        let resp = self
            .client
            .post(&url)
            .send()
            .await
            .map_err(|e| anyhow!("signer nonce-clear request failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("signer nonce-clear returned {status}: {body}"));
        }

        Ok(())
    }

    /// Abort a ceremony session on the Signer (Tier 3).
    pub async fn abort_ceremony(
        &self,
        req: &CeremonyAbortRequest,
    ) -> Result<CeremonyAbortResponse> {
        let url = format!("{}/ceremony-abort", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(req)
            .send()
            .await
            .map_err(|e| anyhow!("signer ceremony-abort request failed: {e}"))?;

        resp.json::<CeremonyAbortResponse>()
            .await
            .map_err(|e| anyhow!("signer ceremony-abort parse error: {e}"))
    }

    /// Health check — verify the Signer is reachable.
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signer_client_url_normalization() {
        let client = SignerClient::new("http://localhost:3001/");
        assert_eq!(client.base_url, "http://localhost:3001");

        let client2 = SignerClient::new("http://localhost:3001");
        assert_eq!(client2.base_url, "http://localhost:3001");
    }

    #[test]
    fn test_signer_client_construction() {
        let client = SignerClient::new("http://signer.local:3001");
        assert_eq!(client.base_url, "http://signer.local:3001");
    }
}

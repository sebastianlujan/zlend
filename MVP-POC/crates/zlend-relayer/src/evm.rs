use alloy::{
    primitives::{Address, FixedBytes, U256},
    providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
    sol,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EvmError {
    #[error("invalid private key: {0}")]
    InvalidKey(String),
    #[error("invalid address: {0}")]
    InvalidAddress(String),
    #[error("transaction failed: {0}")]
    TransactionFailed(String),
    #[error("provider error: {0}")]
    ProviderError(String),
}

// Generate contract bindings from the ABI
sol! {
    #[sol(rpc)]
    contract ZLendMVP {
        function borrow(
            address borrower,
            uint256 collateralZat,
            uint256 amount,
            bytes32 positionId
        ) external;

        function getLoan(bytes32 positionId) external view returns (
            bytes32 positionId_,
            uint256 collateralZat,
            uint256 borrowedAmount,
            address borrower,
            bool active
        );
    }
}

/// EVM signer configuration. Stores the raw params and creates
/// provider + contract instances on each call to avoid complex
/// generic type signatures.
pub struct EvmSigner {
    private_key: String,
    rpc_url: String,
    contract_address: Address,
}

impl EvmSigner {
    pub async fn new(
        private_key: &str,
        rpc_url: &str,
        contract_address: &str,
    ) -> Result<Self, EvmError> {
        // Validate private key
        let _: PrivateKeySigner = private_key
            .parse()
            .map_err(|e| EvmError::InvalidKey(format!("{}", e)))?;

        // Validate contract address
        let addr: Address = contract_address
            .parse()
            .map_err(|e| EvmError::InvalidAddress(format!("{}", e)))?;

        // Validate RPC URL
        let _: url::Url = rpc_url
            .parse()
            .map_err(|e| EvmError::ProviderError(format!("{}", e)))?;

        Ok(Self {
            private_key: private_key.to_string(),
            rpc_url: rpc_url.to_string(),
            contract_address: addr,
        })
    }

    /// Submit a borrow transaction to the ZLendMVP contract.
    ///
    /// This calls `borrow(borrower, collateralZat, amount, positionId)` on the contract.
    /// The relayer's EVM address must be set as `trustedRelayer` in the contract.
    pub async fn borrow(
        &self,
        borrower: &str,
        collateral_zat: u64,
        borrow_amount: u64,
        position_id: &str,
    ) -> Result<String, EvmError> {
        let borrower_addr: Address = borrower
            .parse()
            .map_err(|e| EvmError::InvalidAddress(format!("invalid borrower address: {}", e)))?;

        // Convert position ID (UUID string) to bytes32
        let id_bytes = {
            let mut buf = [0u8; 32];
            let id_clean = position_id.replace('-', "");
            let decoded = hex::decode(&id_clean).unwrap_or_else(|_| {
                let mut raw = position_id.as_bytes().to_vec();
                raw.resize(32, 0);
                raw
            });
            let len = decoded.len().min(32);
            buf[..len].copy_from_slice(&decoded[..len]);
            FixedBytes::<32>::from(buf)
        };

        // Build provider with wallet for this call
        let signer: PrivateKeySigner = self.private_key
            .parse()
            .map_err(|e| EvmError::InvalidKey(format!("{}", e)))?;

        let rpc_url: url::Url = self.rpc_url
            .parse()
            .map_err(|e| EvmError::ProviderError(format!("{}", e)))?;

        let provider = ProviderBuilder::new()
            .wallet(signer)
            .connect_http(rpc_url);

        let contract = ZLendMVP::new(self.contract_address, &provider);

        let tx_hash = contract.borrow(
            borrower_addr,
            U256::from(collateral_zat),
            U256::from(borrow_amount),
            id_bytes,
        )
            .send()
            .await
            .map_err(|e| EvmError::TransactionFailed(format!("send failed: {}", e)))?
            .watch()
            .await
            .map_err(|e| EvmError::TransactionFailed(format!("watch failed: {}", e)))?;

        let tx_hash = format!("{:#x}", tx_hash);
        tracing::info!("Borrow tx confirmed: {}", tx_hash);

        Ok(tx_hash)
    }
}

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "zlend-cli")]
#[command(about = "ZLend CLI — privacy-preserving cross-chain lending")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new ZLend identity (mnemonic + keys + address)
    Generate {
        /// Use mainnet instead of testnet
        #[arg(long, default_value_t = false)]
        mainnet: bool,
    },
    /// Register this identity with a ZLend relayer
    Register {
        /// Relayer URL
        #[arg(long, default_value = "http://localhost:3000")]
        relayer: String,
    },
    /// Check staked ZEC balance and borrow status
    Balance {
        /// Relayer URL
        #[arg(long, default_value = "http://localhost:3000")]
        relayer: String,
    },
    /// Scan a specific Zcash transaction for deposits to this address
    Scan {
        /// Relayer URL
        #[arg(long, default_value = "http://localhost:3000")]
        relayer: String,
        /// Zcash transaction ID to scan
        #[arg(long)]
        txid: String,
    },
    /// Borrow ERC-20 tokens against staked ZEC collateral
    Borrow {
        /// Relayer URL
        #[arg(long, default_value = "http://localhost:3000")]
        relayer: String,
        /// Amount to borrow (in token units, e.g. 100 for 100 mUSDC)
        #[arg(long)]
        amount: u64,
        /// EVM address to receive borrowed tokens
        #[arg(long)]
        recipient: String,
    },
}

/// Local key storage format
#[derive(Serialize, Deserialize)]
struct LocalKeys {
    mnemonic: String,
    sk: String,
    fvk: String,
    ivk: String,
    address: String,
    /// Position ID assigned by the relayer after registration
    position_id: Option<String>,
    /// Relayer URL used for registration
    relayer_url: Option<String>,
}

/// Relayer registration response
#[derive(Deserialize)]
struct RegisterResponse {
    id: String,
    address: String,
}

/// Relayer balance response
#[derive(Deserialize)]
struct BalanceResponse {
    id: String,
    address: String,
    balance_zat: i64,
    borrowed_zat: i64,
    note_count: i64,
}

/// Relayer scan response
#[derive(Deserialize)]
struct ScanResponse {
    found: bool,
    value_zat: Option<u64>,
    total_balance_zat: Option<i64>,
}

/// Relayer borrow response
#[derive(Deserialize)]
struct BorrowResponse {
    success: bool,
    tx_hash: Option<String>,
    borrowed: Option<u64>,
    error: Option<String>,
}

fn keys_path() -> PathBuf {
    let home = dirs::home_dir().expect("Could not determine home directory");
    home.join(".zlend").join("keys.json")
}

fn save_keys(keys: &LocalKeys) -> Result<(), Box<dyn std::error::Error>> {
    let path = keys_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(keys)?;
    std::fs::write(&path, json)?;
    println!("Keys saved to {}", path.display());
    Ok(())
}

fn load_keys() -> Result<LocalKeys, Box<dyn std::error::Error>> {
    let path = keys_path();
    let json = std::fs::read_to_string(&path)
        .map_err(|_| format!("No keys found at {}. Run `zlend-cli generate` first.", path.display()))?;
    let keys: LocalKeys = serde_json::from_str(&json)?;
    Ok(keys)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { mainnet } => {
            let network = if mainnet {
                zlend_core::keys::mainnet()
            } else {
                zlend_core::keys::testnet()
            };

            println!("Generating ZLend identity...\n");
            let keys = zlend_core::keys::generate_keys(&network)?;

            println!("Mnemonic (SAVE THIS — needed for recovery):");
            println!("  {}\n", keys.mnemonic);
            println!("Address (send ZEC here):");
            println!("  {}\n", keys.address);
            println!("Spending Key (hex):");
            println!("  {}...{}\n", &keys.sk[..8], &keys.sk[keys.sk.len()-8..]);
            println!("Full Viewing Key (hex):");
            println!("  {}...{}\n", &keys.fvk[..8], &keys.fvk[keys.fvk.len()-8..]);

            let local = LocalKeys {
                mnemonic: keys.mnemonic.clone(),
                sk: keys.sk.clone(),
                fvk: keys.fvk.clone(),
                ivk: keys.ivk.clone(),
                address: keys.address.clone(),
                position_id: None,
                relayer_url: None,
            };
            save_keys(&local)?;
        }

        Commands::Register { relayer } => {
            let mut keys = load_keys()?;

            println!("Registering with relayer at {}...", relayer);

            let client = reqwest::Client::new();
            let resp = client
                .post(format!("{}/register", relayer))
                .json(&serde_json::json!({
                    "sk": keys.sk,
                    "vk": keys.fvk,
                    "ivk": keys.ivk,
                    "address": keys.address,
                }))
                .send()
                .await?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                eprintln!("Registration failed ({}): {}", status, body);
                return Ok(());
            }

            let reg: RegisterResponse = resp.json().await?;
            println!("\nRegistered successfully!");
            println!("  Position ID: {}", reg.id);
            println!("  Address: {}", reg.address);

            // Save position ID for future commands
            keys.position_id = Some(reg.id);
            keys.relayer_url = Some(relayer);
            save_keys(&keys)?;
        }

        Commands::Balance { relayer } => {
            let keys = load_keys()?;
            let position_id = keys.position_id.as_ref()
                .ok_or("Not registered. Run `zlend-cli register` first.")?;

            let client = reqwest::Client::new();
            let resp = client
                .get(format!("{}/balance/{}", relayer, position_id))
                .send()
                .await?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                eprintln!("Balance query failed ({}): {}", status, body);
                return Ok(());
            }

            let balance: BalanceResponse = resp.json().await?;
            let zec = balance.balance_zat as f64 / 100_000_000.0;
            let borrowed = balance.borrowed_zat as f64 / 1_000_000.0; // mUSDC has 6 decimals

            println!("ZLend Position: {}", balance.id);
            println!("  Address:  {}", balance.address);
            println!("  Staked:   {:.8} ZEC ({} zatoshis, {} notes)", zec, balance.balance_zat, balance.note_count);
            println!("  Borrowed: {:.2} mUSDC ({} units)", borrowed, balance.borrowed_zat);
        }

        Commands::Scan { relayer, txid } => {
            let keys = load_keys()?;
            let position_id = keys.position_id.as_ref()
                .ok_or("Not registered. Run `zlend-cli register` first.")?;

            println!("Scanning transaction {}...", txid);

            let client = reqwest::Client::new();
            let resp = client
                .post(format!("{}/scan/{}", relayer, position_id))
                .json(&serde_json::json!({ "txid": txid }))
                .send()
                .await?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                eprintln!("Scan failed ({}): {}", status, body);
                return Ok(());
            }

            let scan: ScanResponse = resp.json().await?;
            if scan.found {
                let zec = scan.value_zat.unwrap_or(0) as f64 / 100_000_000.0;
                let total = scan.total_balance_zat.unwrap_or(0) as f64 / 100_000_000.0;
                println!("\nDeposit found!");
                println!("  Note value: {:.8} ZEC ({} zatoshis)", zec, scan.value_zat.unwrap_or(0));
                println!("  Total balance: {:.8} ZEC", total);
            } else {
                println!("\nNo deposit found for this address in transaction {}", txid);
            }
        }

        Commands::Borrow { relayer, amount, recipient } => {
            let keys = load_keys()?;
            let position_id = keys.position_id.as_ref()
                .ok_or("Not registered. Run `zlend-cli register` first.")?;

            // Amount is in mUSDC units (6 decimals), so 100 = 100 mUSDC
            let amount_units = amount * 1_000_000; // Convert to base units

            println!("Requesting borrow of {} mUSDC to {}...", amount, recipient);

            let client = reqwest::Client::new();
            let resp = client
                .post(format!("{}/borrow/{}", relayer, position_id))
                .json(&serde_json::json!({
                    "amount": amount_units,
                    "recipient": recipient,
                }))
                .send()
                .await?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                eprintln!("Borrow failed ({}): {}", status, body);
                return Ok(());
            }

            let borrow: BorrowResponse = resp.json().await?;
            if borrow.success {
                println!("\nBorrow successful!");
                println!("  Amount: {} mUSDC", amount);
                if let Some(hash) = &borrow.tx_hash {
                    println!("  Tx hash: {}", hash);
                }
            } else {
                println!("\nBorrow failed: {}", borrow.error.unwrap_or_default());
            }
        }
    }

    Ok(())
}

// Phase 4: OGBank CLI
//
// Commands: generate, register, scan, borrow, balance
//
// Reference: docs/technical/08_mvp.md §Full Sequence

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use ogbank_core::keys::OGBankKeys;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// OGBank CLI — privacy-preserving cross-chain lending.
#[derive(Parser)]
#[command(name = "ogbank-cli", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new OGBank identity (mnemonic + keys).
    Generate {
        /// Optional: restore from existing mnemonic phrase.
        #[arg(long)]
        mnemonic: Option<String>,
    },
    /// Register this identity with a relayer.
    Register {
        /// Relayer base URL (e.g. http://localhost:3000).
        #[arg(long)]
        relayer: String,
    },
    /// Scan a transaction for deposits to our address.
    Scan {
        /// Relayer base URL.
        #[arg(long)]
        relayer: String,
        /// Zcash transaction ID to scan.
        #[arg(long)]
        txid: String,
    },
    /// Borrow tokens against deposited ZEC collateral.
    Borrow {
        /// Relayer base URL.
        #[arg(long)]
        relayer: String,
        /// Amount to borrow (in token smallest units).
        #[arg(long)]
        amount: u64,
        /// EVM recipient address for borrowed tokens.
        #[arg(long)]
        recipient: String,
    },
    /// Check current balance and position status.
    Balance {
        /// Relayer base URL.
        #[arg(long)]
        relayer: String,
    },
}

/// Keyfile stored at ~/.ogbank/keys.json.
#[derive(Serialize, Deserialize)]
struct KeyFile {
    mnemonic: String,
    position_id: String,
    #[serde(with = "hex")]
    sk: Vec<u8>,
    #[serde(with = "hex")]
    fvk: Vec<u8>,
    #[serde(with = "hex")]
    ivk: Vec<u8>,
    #[serde(with = "hex")]
    address: Vec<u8>,
}

fn keyfile_path() -> PathBuf {
    dirs_or_default().join("keys.json")
}

fn dirs_or_default() -> PathBuf {
    let home = std::env::var("OGBANK_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".ogbank")
        });
    home
}

fn load_keys() -> Result<KeyFile> {
    let path = keyfile_path();
    let data = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read keyfile at {}", path.display()))?;
    let kf: KeyFile = serde_json::from_str(&data).context("failed to parse keyfile")?;
    Ok(kf)
}

fn save_keys(kf: &KeyFile) -> Result<()> {
    let path = keyfile_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory {}", parent.display()))?;
    }
    let data = serde_json::to_string_pretty(kf).context("failed to serialize keyfile")?;
    std::fs::write(&path, data)
        .with_context(|| format!("failed to write keyfile at {}", path.display()))?;
    Ok(())
}

fn cmd_generate(mnemonic: Option<String>) -> Result<()> {
    let keys = match mnemonic {
        Some(m) => OGBankKeys::from_mnemonic(&m).context("invalid mnemonic")?,
        None => OGBankKeys::generate().context("key generation failed")?,
    };

    let position_id = uuid::Uuid::new_v4().to_string();
    let kf = KeyFile {
        mnemonic: keys.mnemonic.clone(),
        position_id: position_id.clone(),
        sk: keys.sk_bytes().to_vec(),
        fvk: keys.fvk_bytes().to_vec(),
        ivk: keys.ivk_bytes().to_vec(),
        address: keys.address.to_raw_address_bytes().to_vec(),
    };

    save_keys(&kf)?;

    println!("Identity generated successfully.");
    println!("Position ID: {position_id}");
    println!("Address: {}", hex::encode(&kf.address));
    println!("Mnemonic (BACK THIS UP):\n  {}", keys.mnemonic);
    println!("\nKeyfile saved to: {}", keyfile_path().display());

    Ok(())
}

async fn cmd_register(relayer: &str) -> Result<()> {
    let kf = load_keys().context("run 'ogbank-cli generate' first")?;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{relayer}/register"))
        .json(&serde_json::json!({
            "position_id": kf.position_id,
            "sk": hex::encode(&kf.sk),
            "fvk": hex::encode(&kf.fvk),
            "ivk": hex::encode(&kf.ivk),
            "address": hex::encode(&kf.address),
        }))
        .send()
        .await
        .context("failed to reach relayer")?;

    if resp.status().is_success() {
        println!("Registered with relayer. Position ID: {}", kf.position_id);
    } else {
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("registration failed: {body}");
    }

    Ok(())
}

async fn cmd_scan(relayer: &str, txid: &str) -> Result<()> {
    let kf = load_keys().context("run 'ogbank-cli generate' first")?;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{relayer}/scan/{}", kf.position_id))
        .json(&serde_json::json!({ "txid": txid }))
        .send()
        .await
        .context("failed to reach relayer")?;

    let body: serde_json::Value = resp.json().await.context("invalid response")?;
    println!("{}", serde_json::to_string_pretty(&body)?);

    Ok(())
}

async fn cmd_borrow(relayer: &str, amount: u64, recipient: &str) -> Result<()> {
    if amount == 0 {
        anyhow::bail!("borrow amount must be > 0");
    }

    let kf = load_keys().context("run 'ogbank-cli generate' first")?;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{relayer}/borrow/{}", kf.position_id))
        .json(&serde_json::json!({
            "amount": amount,
            "recipient": recipient,
        }))
        .send()
        .await
        .context("failed to reach relayer")?;

    if resp.status().is_success() {
        let body: serde_json::Value = resp.json().await.context("invalid response")?;
        println!("Borrow successful:");
        println!("{}", serde_json::to_string_pretty(&body)?);
    } else {
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("borrow failed: {body}");
    }

    Ok(())
}

async fn cmd_balance(relayer: &str) -> Result<()> {
    let kf = load_keys().context("run 'ogbank-cli generate' first")?;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{relayer}/balance/{}", kf.position_id))
        .send()
        .await
        .context("failed to reach relayer")?;

    let body: serde_json::Value = resp.json().await.context("invalid response")?;
    println!("{}", serde_json::to_string_pretty(&body)?);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { mnemonic } => cmd_generate(mnemonic),
        Commands::Register { relayer } => cmd_register(&relayer).await,
        Commands::Scan { relayer, txid } => cmd_scan(&relayer, &txid).await,
        Commands::Borrow {
            relayer,
            amount,
            recipient,
        } => cmd_borrow(&relayer, amount, &recipient).await,
        Commands::Balance { relayer } => cmd_balance(&relayer).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

    fn with_temp_home<F: FnOnce()>(f: F) {
        let dir = tempfile::tempdir().expect("temp dir");
        env::set_var("OGBANK_HOME", dir.path());
        f();
        env::remove_var("OGBANK_HOME");
    }

    #[test]
    fn test_generate_creates_keyfile() {
        with_temp_home(|| {
            cmd_generate(Some(TEST_MNEMONIC.to_string())).expect("generate failed");
            let path = keyfile_path();
            assert!(path.exists(), "keyfile must be created");

            let kf = load_keys().expect("load failed");
            assert_eq!(kf.sk.len(), 32, "sk must be 32 bytes");
            assert_eq!(kf.fvk.len(), 96, "fvk must be 96 bytes");
            assert_eq!(kf.ivk.len(), 64, "ivk must be 64 bytes");
            assert_eq!(kf.address.len(), 43, "address must be 43 bytes");
        });
    }

    #[test]
    fn test_generate_deterministic() {
        with_temp_home(|| {
            cmd_generate(Some(TEST_MNEMONIC.to_string())).expect("generate 1 failed");
            let kf1 = load_keys().unwrap();

            cmd_generate(Some(TEST_MNEMONIC.to_string())).expect("generate 2 failed");
            let kf2 = load_keys().unwrap();

            assert_eq!(kf1.sk, kf2.sk, "same mnemonic must produce same sk");
            assert_eq!(kf1.fvk, kf2.fvk, "same mnemonic must produce same fvk");
            assert_eq!(kf1.address, kf2.address, "same mnemonic must produce same address");
        });
    }

    #[test]
    fn test_generate_random() {
        with_temp_home(|| {
            cmd_generate(None).expect("random generate failed");
            let kf = load_keys().unwrap();
            let words: Vec<&str> = kf.mnemonic.split_whitespace().collect();
            assert_eq!(words.len(), 24, "mnemonic must be 24 words");
        });
    }

    #[test]
    fn test_borrow_validates_amount() {
        // cmd_borrow with amount=0 should fail without needing a server
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(cmd_borrow("http://localhost:9999", 0, "0xAlice"));
        assert!(result.is_err(), "zero borrow amount must be rejected");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("must be > 0"),
            "error must mention amount: {err}"
        );
    }

    #[test]
    fn test_keyfile_roundtrip() {
        with_temp_home(|| {
            let kf = KeyFile {
                mnemonic: "test words".to_string(),
                position_id: "pos-123".to_string(),
                sk: vec![1; 32],
                fvk: vec![2; 96],
                ivk: vec![3; 64],
                address: vec![4; 43],
            };
            save_keys(&kf).expect("save failed");
            let loaded = load_keys().expect("load failed");

            assert_eq!(loaded.mnemonic, kf.mnemonic);
            assert_eq!(loaded.position_id, kf.position_id);
            assert_eq!(loaded.sk, kf.sk);
            assert_eq!(loaded.fvk, kf.fvk);
            assert_eq!(loaded.ivk, kf.ivk);
            assert_eq!(loaded.address, kf.address);
        });
    }
}

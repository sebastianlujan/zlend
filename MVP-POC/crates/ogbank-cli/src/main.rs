// Phase 4: OGBank CLI
//
// Commands: generate, register, scan, borrow, balance,
//           vault-create, ticket-create, ceremony-run
//
// Reference: docs/technical/08_mvp.md §Full Sequence
//            docs/technical/11_rfc-ogb-001.md §4.3, §5, §7

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
    /// Create a new FROST 2-of-3 vault via DKG (RFC §4.3).
    VaultCreate {
        /// Vault nonce (default: 0). Increment for multiple vaults under same seed.
        #[arg(long, default_value_t = 0)]
        vault_nonce: u32,
    },
    /// Create an authorization ticket batch (RFC §5.1–5.6).
    TicketCreate {
        /// Maximum zatoshi authorized per ticket.
        #[arg(long)]
        max_amount: u64,
        /// Recipient address (hex-encoded bytes).
        #[arg(long)]
        destination: String,
        /// Block height after which tickets expire.
        #[arg(long)]
        expiry_block: u32,
        /// Number of tickets to create in this batch.
        #[arg(long, default_value_t = 1)]
        count: usize,
    },
    /// Run a local end-to-end signing ceremony demo (RFC §7).
    CeremonyRun {
        /// Spend amount in zatoshi.
        #[arg(long, default_value_t = 1_000_000)]
        amount: u64,
        /// Recipient address (hex or plaintext).
        #[arg(long, default_value = "demo-recipient")]
        destination: String,
        /// Expiry block height.
        #[arg(long, default_value_t = 1_000_000)]
        expiry_block: u32,
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

/// Vault file stored at ~/.ogbank/vault.json.
#[derive(Serialize, Deserialize)]
struct VaultFile {
    vault_id: String,
    vault_nonce: u32,
    group_public_key: String,
    participant_count: usize,
}

/// Ticket batch file stored at ~/.ogbank/tickets/<batch>.json.
#[derive(Serialize, Deserialize)]
struct TicketBatchFile {
    vault_id: String,
    tree_root: String,
    tickets: Vec<TicketEntry>,
}

#[derive(Serialize, Deserialize)]
struct TicketEntry {
    auth_secret: String,
    auth_nullifier_hash: String,
    commitment: String,
    max_amount: u64,
    destination_hash: String,
    expiry_block: u32,
    leaf_index: u32,
}

fn ogbank_home() -> PathBuf {
    std::env::var("OGBANK_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".ogbank")
        })
}

fn load_keys() -> Result<KeyFile> {
    load_keys_from(&ogbank_home())
}

fn load_keys_from(home: &std::path::Path) -> Result<KeyFile> {
    let path = home.join("keys.json");
    let data = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read keyfile at {}", path.display()))?;
    let kf: KeyFile = serde_json::from_str(&data).context("failed to parse keyfile")?;
    Ok(kf)
}

fn save_keys_to(kf: &KeyFile, home: &std::path::Path) -> Result<()> {
    let path = home.join("keys.json");
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
    cmd_generate_in(mnemonic, &ogbank_home())
}

fn cmd_generate_in(mnemonic: Option<String>, home: &std::path::Path) -> Result<()> {
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

    save_keys_to(&kf, home)?;

    let keyfile = home.join("keys.json");
    println!("Identity generated successfully.");
    println!("Position ID: {position_id}");
    println!("Address: {}", hex::encode(&kf.address));
    println!("Mnemonic (BACK THIS UP):\n  {}", keys.mnemonic);
    println!("\nKeyfile saved to: {}", keyfile.display());

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

// ---------------------------------------------------------------------------
// Vault commands (RFC-OGB-001 §4.3, §5, §7)
// ---------------------------------------------------------------------------

fn cmd_vault_create(vault_nonce: u32) -> Result<()> {
    cmd_vault_create_in(vault_nonce, &ogbank_home())
}

fn cmd_vault_create_in(vault_nonce: u32, home: &std::path::Path) -> Result<()> {
    use ogbank_core::frost;

    let mut rng = rand::rngs::OsRng;
    let vault = frost::run_dkg_local(&mut rng)
        .map_err(|e| anyhow::anyhow!("DKG failed: {e}"))?;

    let ak = frost::group_public_key_bytes(&vault.public_key_package);
    let vault_id = frost::compute_vault_id(&ak, vault_nonce);

    let vf = VaultFile {
        vault_id: hex::encode(vault_id),
        vault_nonce,
        group_public_key: hex::encode(ak),
        participant_count: vault.key_packages.len(),
    };

    let path = home.join("vault.json");
    std::fs::create_dir_all(home)
        .with_context(|| format!("failed to create directory {}", home.display()))?;
    let data = serde_json::to_string_pretty(&vf).context("failed to serialize vault")?;
    std::fs::write(&path, &data)
        .with_context(|| format!("failed to write vault at {}", path.display()))?;

    println!("Vault created successfully (FROST 2-of-3 DKG).");
    println!("Vault ID:          {}", vf.vault_id);
    println!("Group Public Key:  {}", vf.group_public_key);
    println!("Participants:      {}", vf.participant_count);
    println!("Vault Nonce:       {vault_nonce}");
    println!("\nVault saved to: {}", path.display());

    Ok(())
}

fn cmd_ticket_create(
    max_amount: u64,
    destination: &str,
    expiry_block: u32,
    count: usize,
) -> Result<()> {
    cmd_ticket_create_in(max_amount, destination, expiry_block, count, &ogbank_home())
}

fn cmd_ticket_create_in(
    max_amount: u64,
    destination: &str,
    expiry_block: u32,
    count: usize,
    home: &std::path::Path,
) -> Result<()> {
    use ogbank_core::auth::{AuthTree, AuthorizationTicket};

    // Load vault to get vault_id
    let vault_path = home.join("vault.json");
    let vault_data = std::fs::read_to_string(&vault_path)
        .with_context(|| format!("no vault found at {} — run 'vault-create' first", vault_path.display()))?;
    let vf: VaultFile = serde_json::from_str(&vault_data).context("failed to parse vault.json")?;

    let dest_bytes = hex::decode(destination)
        .unwrap_or_else(|_| destination.as_bytes().to_vec());

    let mut tree = AuthTree::new();
    let mut tickets = Vec::with_capacity(count);

    for _ in 0..count {
        let mut auth_secret = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut auth_secret);

        let ticket = AuthorizationTicket::new(auth_secret, max_amount, &dest_bytes, expiry_block);
        let commitment = ticket.commitment();
        let leaf_index = tree.insert(commitment)
            .map_err(|e| anyhow::anyhow!("tree insert failed: {e}"))?;

        tickets.push((ticket, commitment, leaf_index));
    }

    let tree_root = tree.root();

    let entries: Vec<TicketEntry> = tickets
        .iter()
        .map(|(ticket, commitment, leaf_index)| {
            TicketEntry {
                auth_secret: hex::encode(ticket.auth_secret),
                auth_nullifier_hash: hex::encode(ticket.nullifier_hash()),
                commitment: hex::encode(commitment),
                max_amount: ticket.max_amount,
                destination_hash: hex::encode(ticket.destination_hash),
                expiry_block: ticket.expiry_block,
                leaf_index: *leaf_index,
            }
        })
        .collect();

    let batch = TicketBatchFile {
        vault_id: vf.vault_id.clone(),
        tree_root: hex::encode(tree_root),
        tickets: entries,
    };

    let tickets_dir = home.join("tickets");
    std::fs::create_dir_all(&tickets_dir)?;

    // Find next batch number
    let batch_num = std::fs::read_dir(&tickets_dir)
        .map(|rd| rd.count())
        .unwrap_or(0);
    let batch_path = tickets_dir.join(format!("batch_{batch_num}.json"));

    let data = serde_json::to_string_pretty(&batch).context("failed to serialize ticket batch")?;
    std::fs::write(&batch_path, &data)?;

    println!("Authorization ticket batch created.");
    println!("Vault ID:     {}", vf.vault_id);
    println!("Tickets:      {count}");
    println!("Max Amount:   {max_amount} zat");
    println!("Expiry Block: {expiry_block}");
    println!("Tree Root:    {}", hex::encode(tree_root));
    println!("\nBatch saved to: {}", batch_path.display());

    Ok(())
}

fn cmd_ceremony_run(amount: u64, destination: &str, expiry_block: u32) -> Result<()> {
    use ogbank_core::auth::{AuthTree, AuthorizationTicket};
    use ogbank_core::ceremony;
    use ogbank_core::frost::{self, Identifier};
    use ogbank_core::signer::{
        compute_sighash, AuthorizationProof, ProposedTx, SignerState, SigningRequest,
    };

    let mut rng = rand::rngs::OsRng;

    // 1. FROST DKG
    let vault = frost::run_dkg_local(&mut rng)
        .map_err(|e| anyhow::anyhow!("DKG failed: {e}"))?;
    let ak = frost::group_public_key_bytes(&vault.public_key_package);
    let vault_id = frost::compute_vault_id(&ak, 0);
    println!("Vault ID: {}", hex::encode(vault_id));

    // 2. Create authorization ticket
    let dest_bytes = destination.as_bytes();
    let mut auth_secret = [0u8; 32];
    rand::RngCore::fill_bytes(&mut rng, &mut auth_secret);
    let ticket = AuthorizationTicket::new(auth_secret, amount, dest_bytes, expiry_block);

    // 3. Build auth tree
    let mut tree = AuthTree::new();
    let leaf_index = tree.insert(ticket.commitment())
        .map_err(|e| anyhow::anyhow!("tree insert failed: {e}"))?;
    let proof = tree.proof(leaf_index)
        .map_err(|e| anyhow::anyhow!("proof failed: {e}"))?;
    let tree_root = tree.root();

    // 4. Build signing request
    let tx_data = b"ceremony-demo-tx-data".to_vec();
    let sighash = compute_sighash(&tx_data);

    let proposed_tx = ProposedTx {
        total_spend_value: amount,
        recipient_address: dest_bytes.to_vec(),
        tx_data,
    };

    let auth_proof = AuthorizationProof {
        auth_secret: ticket.auth_secret,
        auth_nullifier_hash: ticket.nullifier_hash(),
        max_amount: ticket.max_amount,
        destination_hash: ticket.destination_hash,
        expiry_block: ticket.expiry_block,
        merkle_proof: proof,
    };

    let request = SigningRequest {
        sighash,
        auth_proof,
        proposed_tx,
    };

    // 5. Run ceremony (Owner A + Relayer)
    let mut signer_state = SignerState::new(tree_root, 100);
    let signer_ids = [
        Identifier::try_from(frost::OWNER_A_ID).unwrap(),
        Identifier::try_from(frost::RELAYER_ID).unwrap(),
    ];

    let result = ceremony::run_ceremony_local(&vault, &signer_ids, &request, &mut signer_state, &mut rng)
        .map_err(|e| anyhow::anyhow!("ceremony failed: {e:?}"))?;

    // 6. Verify
    let verified = result
        .randomized_verifying_key
        .verify(&sighash, &result.signature);

    println!("Signature:    {}", hex::encode(result.signature.serialize()));
    match verified {
        Ok(()) => println!("Verification: PASSED"),
        Err(e) => println!("Verification: FAILED ({e})"),
    }
    println!("\nEnd-to-end ceremony complete: DKG -> ticket -> validation -> FROST sign -> verify");

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
        Commands::VaultCreate { vault_nonce } => cmd_vault_create(vault_nonce),
        Commands::TicketCreate {
            max_amount,
            destination,
            expiry_block,
            count,
        } => cmd_ticket_create(max_amount, &destination, expiry_block, count),
        Commands::CeremonyRun {
            amount,
            destination,
            expiry_block,
        } => cmd_ceremony_run(amount, &destination, expiry_block),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

    #[test]
    fn test_generate_creates_keyfile() {
        let dir = tempfile::tempdir().expect("temp dir");
        let home = dir.path();

        cmd_generate_in(Some(TEST_MNEMONIC.to_string()), home).expect("generate failed");
        assert!(home.join("keys.json").exists(), "keyfile must be created");

        let kf = load_keys_from(home).expect("load failed");
        assert_eq!(kf.sk.len(), 32, "sk must be 32 bytes");
        assert_eq!(kf.fvk.len(), 96, "fvk must be 96 bytes");
        assert_eq!(kf.ivk.len(), 64, "ivk must be 64 bytes");
        assert_eq!(kf.address.len(), 43, "address must be 43 bytes");
    }

    #[test]
    fn test_generate_deterministic() {
        let dir = tempfile::tempdir().expect("temp dir");
        let home = dir.path();

        cmd_generate_in(Some(TEST_MNEMONIC.to_string()), home).expect("generate 1 failed");
        let kf1 = load_keys_from(home).unwrap();

        cmd_generate_in(Some(TEST_MNEMONIC.to_string()), home).expect("generate 2 failed");
        let kf2 = load_keys_from(home).unwrap();

        assert_eq!(kf1.sk, kf2.sk, "same mnemonic must produce same sk");
        assert_eq!(kf1.fvk, kf2.fvk, "same mnemonic must produce same fvk");
        assert_eq!(kf1.address, kf2.address, "same mnemonic must produce same address");
    }

    #[test]
    fn test_generate_random() {
        let dir = tempfile::tempdir().expect("temp dir");
        let home = dir.path();

        cmd_generate_in(None, home).expect("random generate failed");
        let kf = load_keys_from(home).unwrap();
        let words: Vec<&str> = kf.mnemonic.split_whitespace().collect();
        assert_eq!(words.len(), 24, "mnemonic must be 24 words");
    }

    #[test]
    fn test_borrow_validates_amount() {
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
    fn test_vault_create() {
        let dir = tempfile::tempdir().expect("temp dir");
        let home = dir.path();

        cmd_vault_create_in(0, home).expect("vault create failed");
        assert!(home.join("vault.json").exists(), "vault.json must be created");

        let data = std::fs::read_to_string(home.join("vault.json")).unwrap();
        let vf: VaultFile = serde_json::from_str(&data).unwrap();

        assert_eq!(vf.vault_nonce, 0);
        assert_eq!(vf.participant_count, 3, "must have 3 participants");
        assert_eq!(vf.vault_id.len(), 64, "vault_id must be 32 bytes hex");
        assert_eq!(vf.group_public_key.len(), 64, "group key must be 32 bytes hex");
    }

    #[test]
    fn test_ticket_create() {
        let dir = tempfile::tempdir().expect("temp dir");
        let home = dir.path();

        // Create vault first
        cmd_vault_create_in(0, home).expect("vault create failed");

        // Create ticket batch
        cmd_ticket_create_in(
            1_000_000,
            "deadbeef",
            500_000,
            3,
            home,
        )
        .expect("ticket create failed");

        let tickets_dir = home.join("tickets");
        assert!(tickets_dir.exists(), "tickets dir must be created");

        let batch_path = tickets_dir.join("batch_0.json");
        assert!(batch_path.exists(), "batch file must be created");

        let data = std::fs::read_to_string(batch_path).unwrap();
        let batch: TicketBatchFile = serde_json::from_str(&data).unwrap();

        assert_eq!(batch.tickets.len(), 3, "must create 3 tickets");
        assert_eq!(batch.tree_root.len(), 64, "tree root must be 32 bytes hex");
        for t in &batch.tickets {
            assert_eq!(t.max_amount, 1_000_000);
            assert_eq!(t.expiry_block, 500_000);
        }
    }

    #[test]
    fn test_ceremony_run() {
        let result = cmd_ceremony_run(500_000, "test-recipient", 1_000_000);
        assert!(result.is_ok(), "ceremony must succeed: {:?}", result.err());
    }

    #[test]
    fn test_keyfile_roundtrip() {
        let dir = tempfile::tempdir().expect("temp dir");
        let home = dir.path();

        let kf = KeyFile {
            mnemonic: "test words".to_string(),
            position_id: "pos-123".to_string(),
            sk: vec![1; 32],
            fvk: vec![2; 96],
            ivk: vec![3; 64],
            address: vec![4; 43],
        };
        save_keys_to(&kf, home).expect("save failed");
        let loaded = load_keys_from(home).expect("load failed");

        assert_eq!(loaded.mnemonic, kf.mnemonic);
        assert_eq!(loaded.position_id, kf.position_id);
        assert_eq!(loaded.sk, kf.sk);
        assert_eq!(loaded.fvk, kf.fvk);
        assert_eq!(loaded.ivk, kf.ivk);
        assert_eq!(loaded.address, kf.address);
    }
}

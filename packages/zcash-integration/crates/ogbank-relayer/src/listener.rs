// Phase 9: Event Listener — ZEC Withdrawal via z_sendmany
//
// Background task that processes pending withdrawals by sending ZEC
// back to users via the ZcashNodeClient.
//
// In production, FinishPayment events from the EVM contract trigger
// new withdrawal rows in the DB. This listener picks them up and
// calls z_sendmany to return collateral.
//
// Reference: docs/technical/08_mvp.md, docs/technical/12_continuation-plan.md §Step 6

use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use rusqlite::Connection;
use tracing::{error, info, warn};

use crate::db;
use crate::zcash::ZcashNodeClient;

/// Configuration for the withdrawal listener.
pub struct ListenerConfig {
    /// Zcash shielded address to send from (relayer's z-address).
    pub from_address: String,
    /// Poll interval for checking pending withdrawals.
    pub poll_interval: Duration,
    /// Maximum retry attempts per withdrawal before marking as failed.
    pub max_retries: i32,
}

impl Default for ListenerConfig {
    fn default() -> Self {
        Self {
            from_address: String::new(),
            poll_interval: Duration::from_secs(30),
            max_retries: 5,
        }
    }
}

/// Run the withdrawal listener loop.
///
/// Polls the database for pending withdrawals and sends ZEC via z_sendmany.
/// Runs indefinitely — designed to be spawned via `tokio::spawn`.
pub async fn run_withdrawal_listener(
    db: Arc<Mutex<Connection>>,
    zcash_client: ZcashNodeClient,
    config: ListenerConfig,
) {
    info!("withdrawal listener started (poll every {:?})", config.poll_interval);

    loop {
        if let Err(e) = process_pending_withdrawals(&db, &zcash_client, &config).await {
            error!("withdrawal processing error: {e}");
        }

        tokio::time::sleep(config.poll_interval).await;
    }
}

/// Process all pending withdrawals in a single pass.
///
/// Returns the number of withdrawals successfully submitted.
pub async fn process_pending_withdrawals(
    db: &Mutex<Connection>,
    zcash_client: &ZcashNodeClient,
    config: &ListenerConfig,
) -> Result<usize> {
    let pending = {
        let conn = db.lock().unwrap();
        db::get_pending_withdrawals(&conn)?
    };

    if pending.is_empty() {
        return Ok(0);
    }

    info!("processing {} pending withdrawal(s)", pending.len());
    let mut success_count = 0;

    for (wd_id, position_id, amount_zat, recipient_address, retry_count) in &pending {
        // Look up the position's origin z-address for logging
        let _origin = {
            let conn = db.lock().unwrap();
            db::get_position_address(&conn, position_id).unwrap_or_default()
        };

        let result = send_zec_withdrawal(
            zcash_client,
            &config.from_address,
            recipient_address,
            *amount_zat as u64,
        )
        .await;

        let conn = db.lock().unwrap();
        match result {
            Ok(opid) => {
                info!(
                    "withdrawal {} submitted: opid={}, {} zat → {}",
                    wd_id, opid, amount_zat, recipient_address
                );
                db::update_withdrawal_status(&conn, *wd_id, "submitted", Some(&opid))?;
                success_count += 1;
            }
            Err(e) => {
                warn!(
                    "withdrawal {} failed (attempt {}): {e}",
                    wd_id,
                    retry_count + 1
                );
                db::increment_withdrawal_retry(&conn, *wd_id)?;

                // Mark as failed if max retries exceeded
                if *retry_count + 1 >= config.max_retries {
                    error!("withdrawal {} exceeded max retries, marking failed", wd_id);
                    db::update_withdrawal_status(&conn, *wd_id, "failed", None)?;
                }
            }
        }
    }

    Ok(success_count)
}

/// Send ZEC from the relayer's z-address to a recipient.
async fn send_zec_withdrawal(
    zcash_client: &ZcashNodeClient,
    from_address: &str,
    to_address: &str,
    amount_zat: u64,
) -> Result<String> {
    zcash_client
        .z_sendmany(from_address, to_address, amount_zat)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> Arc<Mutex<Connection>> {
        let conn = Connection::open_in_memory().unwrap();
        db::init_db(&conn).unwrap();
        Arc::new(Mutex::new(conn))
    }

    fn seed_withdrawal(db: &Mutex<Connection>, position_id: &str, amount_zat: i64, recipient: &str) -> i64 {
        let conn = db.lock().unwrap();
        db::insert_position(&conn, position_id, "zs1origin", &[1; 96], &[2; 64], &[3; 64]).unwrap();
        let loan_id = db::insert_loan(&conn, position_id, amount_zat, "0xBorrower", Some("0xtx")).unwrap();

        // Use unique nullifiers for each withdrawal
        let borrow_nf: Vec<u8> = position_id.bytes().chain(std::iter::repeat(0)).take(32).collect();
        let repay_nf: Vec<u8> = position_id.bytes().chain(std::iter::repeat(1)).take(32).collect();
        let avax_hash = format!("0xavax_{position_id}");

        db::insert_withdrawal(
            &conn,
            position_id,
            loan_id,
            &avax_hash,
            100,
            &borrow_nf,
            &repay_nf,
            amount_zat,
            recipient,
        )
        .unwrap()
    }

    #[test]
    fn test_get_pending_withdrawals() {
        let db = test_db();
        seed_withdrawal(&db, "pos-1", 1_000_000, "zs1dest1");
        seed_withdrawal(&db, "pos-2", 2_000_000, "zs1dest2");

        let conn = db.lock().unwrap();
        let pending = db::get_pending_withdrawals(&conn).unwrap();
        assert_eq!(pending.len(), 2);
        assert_eq!(pending[0].2, 1_000_000); // amount_zat
        assert_eq!(pending[1].2, 2_000_000);
    }

    #[test]
    fn test_withdrawal_status_update() {
        let db = test_db();
        let wd_id = seed_withdrawal(&db, "pos-1", 500_000, "zs1dest");

        let conn = db.lock().unwrap();
        db::update_withdrawal_status(&conn, wd_id, "submitted", Some("opid-123")).unwrap();

        let status: String = conn
            .query_row(
                "SELECT zec_status FROM withdrawals WHERE id = ?1",
                rusqlite::params![wd_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(status, "submitted");

        let tx_hash: Option<String> = conn
            .query_row(
                "SELECT zec_tx_hash FROM withdrawals WHERE id = ?1",
                rusqlite::params![wd_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(tx_hash, Some("opid-123".to_string()));
    }

    #[test]
    fn test_retry_increment() {
        let db = test_db();
        let wd_id = seed_withdrawal(&db, "pos-1", 500_000, "zs1dest");

        let conn = db.lock().unwrap();
        db::increment_withdrawal_retry(&conn, wd_id).unwrap();
        db::increment_withdrawal_retry(&conn, wd_id).unwrap();

        let retry_count: i32 = conn
            .query_row(
                "SELECT retry_count FROM withdrawals WHERE id = ?1",
                rusqlite::params![wd_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(retry_count, 2);
    }

    #[test]
    fn test_failed_withdrawals_not_picked_up() {
        let db = test_db();
        let wd_id = seed_withdrawal(&db, "pos-1", 500_000, "zs1dest");

        // Mark as failed
        let conn = db.lock().unwrap();
        db::update_withdrawal_status(&conn, wd_id, "failed", None).unwrap();

        let pending = db::get_pending_withdrawals(&conn).unwrap();
        assert!(pending.is_empty(), "failed withdrawals should not be picked up");
    }

    #[test]
    fn test_max_retries_excludes_from_pending() {
        let db = test_db();
        let wd_id = seed_withdrawal(&db, "pos-1", 500_000, "zs1dest");

        let conn = db.lock().unwrap();
        // Bump retry_count to 5 (max)
        for _ in 0..5 {
            db::increment_withdrawal_retry(&conn, wd_id).unwrap();
        }

        let pending = db::get_pending_withdrawals(&conn).unwrap();
        assert!(pending.is_empty(), "withdrawals at max retries should be excluded");
    }

    #[test]
    fn test_listener_config_defaults() {
        let config = ListenerConfig::default();
        assert_eq!(config.poll_interval, Duration::from_secs(30));
        assert_eq!(config.max_retries, 5);
    }
}

// Phase 5: SQLite Database Schema
//
// Tables: positions, notes, loans, withdrawals, event_cursor, vaults
//
// Reference: docs/technical/08_mvp.md §Data Model
//            docs/technical/11_rfc-ogb-001.md §4.3, §7

use rusqlite::{params, Connection, Result as SqliteResult};

/// Initialize the database schema. Creates tables if they don't exist.
pub fn init_db(conn: &Connection) -> SqliteResult<()> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS positions (
            id TEXT PRIMARY KEY,
            address TEXT NOT NULL,
            vk BLOB NOT NULL,
            sk_encrypted BLOB NOT NULL,
            ivk BLOB NOT NULL,
            balance_zat INTEGER NOT NULL DEFAULT 0,
            borrowed_zat INTEGER NOT NULL DEFAULT 0,
            note_count INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            position_id TEXT NOT NULL REFERENCES positions(id),
            txid TEXT NOT NULL,
            value_zat INTEGER NOT NULL,
            nullifier BLOB,
            memo BLOB,
            found_at TEXT NOT NULL DEFAULT (datetime('now')),
            spent INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS loans (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            position_id TEXT NOT NULL REFERENCES positions(id),
            amount INTEGER NOT NULL,
            recipient TEXT NOT NULL,
            tx_hash TEXT,
            status TEXT NOT NULL DEFAULT 'active',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS withdrawals (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            position_id TEXT NOT NULL REFERENCES positions(id),
            loan_id INTEGER NOT NULL REFERENCES loans(id),
            avax_tx_hash TEXT NOT NULL UNIQUE,
            block_number INTEGER NOT NULL,
            borrow_nullifier BLOB NOT NULL UNIQUE,
            repay_nullifier BLOB NOT NULL,
            amount_zat INTEGER NOT NULL,
            recipient_address TEXT NOT NULL,
            zec_tx_hash TEXT,
            zec_status TEXT NOT NULL DEFAULT 'pending',
            retry_count INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS event_cursor (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            last_block INTEGER NOT NULL DEFAULT 0,
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS vaults (
            vault_id TEXT PRIMARY KEY,
            vault_address TEXT NOT NULL,
            group_public_key BLOB NOT NULL,
            status TEXT NOT NULL DEFAULT 'inactive',
            balance_zat INTEGER NOT NULL DEFAULT 0,
            last_scanned_height INTEGER NOT NULL DEFAULT 0,
            notes_received INTEGER NOT NULL DEFAULT 0,
            authorized_spends INTEGER NOT NULL DEFAULT 0,
            unauthorized_spends INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS vault_state_transitions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            vault_id TEXT NOT NULL REFERENCES vaults(vault_id),
            from_state TEXT NOT NULL,
            to_state TEXT NOT NULL,
            reason TEXT,
            block_height INTEGER,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS vault_revocations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            vault_id TEXT NOT NULL REFERENCES vaults(vault_id),
            owner_signature BLOB NOT NULL,
            reason TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        ",
    )?;

    Ok(())
}

// --- Position operations ---

pub fn insert_position(
    conn: &Connection,
    id: &str,
    address: &str,
    vk: &[u8],
    sk_encrypted: &[u8],
    ivk: &[u8],
) -> SqliteResult<()> {
    conn.execute(
        "INSERT INTO positions (id, address, vk, sk_encrypted, ivk) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, address, vk, sk_encrypted, ivk],
    )?;
    Ok(())
}

pub fn get_position_balance(conn: &Connection, id: &str) -> SqliteResult<(i64, i64, i64)> {
    conn.query_row(
        "SELECT balance_zat, borrowed_zat, note_count FROM positions WHERE id = ?1",
        params![id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )
}

pub fn update_balance(conn: &Connection, id: &str, balance_zat: i64) -> SqliteResult<()> {
    conn.execute(
        "UPDATE positions SET balance_zat = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![balance_zat, id],
    )?;
    Ok(())
}

pub fn get_position_ivk(conn: &Connection, id: &str) -> SqliteResult<Vec<u8>> {
    conn.query_row(
        "SELECT ivk FROM positions WHERE id = ?1",
        params![id],
        |row| row.get(0),
    )
}

// --- Note operations ---

pub fn insert_note(
    conn: &Connection,
    position_id: &str,
    txid: &str,
    value_zat: i64,
    nullifier: Option<&[u8]>,
    memo: Option<&[u8]>,
) -> SqliteResult<i64> {
    conn.execute(
        "INSERT INTO notes (position_id, txid, value_zat, nullifier, memo) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![position_id, txid, value_zat, nullifier, memo],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_notes_for_position(
    conn: &Connection,
    position_id: &str,
) -> SqliteResult<Vec<(i64, String, i64, bool)>> {
    let mut stmt =
        conn.prepare("SELECT id, txid, value_zat, spent FROM notes WHERE position_id = ?1")?;
    let rows = stmt.query_map(params![position_id], |row| {
        Ok((
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get::<_, i32>(3)? != 0,
        ))
    })?;
    rows.collect()
}

// --- Loan operations ---

pub fn insert_loan(
    conn: &Connection,
    position_id: &str,
    amount: i64,
    recipient: &str,
    tx_hash: Option<&str>,
) -> SqliteResult<i64> {
    conn.execute(
        "INSERT INTO loans (position_id, amount, recipient, tx_hash) VALUES (?1, ?2, ?3, ?4)",
        params![position_id, amount, recipient, tx_hash],
    )?;
    Ok(conn.last_insert_rowid())
}

// --- Withdrawal operations ---

pub fn insert_withdrawal(
    conn: &Connection,
    position_id: &str,
    loan_id: i64,
    avax_tx_hash: &str,
    block_number: i64,
    borrow_nullifier: &[u8],
    repay_nullifier: &[u8],
    amount_zat: i64,
    recipient_address: &str,
) -> SqliteResult<i64> {
    conn.execute(
        "INSERT INTO withdrawals (position_id, loan_id, avax_tx_hash, block_number, borrow_nullifier, repay_nullifier, amount_zat, recipient_address) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![position_id, loan_id, avax_tx_hash, block_number, borrow_nullifier, repay_nullifier, amount_zat, recipient_address],
    )?;
    Ok(conn.last_insert_rowid())
}

// --- Event cursor operations ---

pub fn get_last_block(conn: &Connection) -> SqliteResult<i64> {
    conn.query_row(
        "SELECT last_block FROM event_cursor WHERE id = 1",
        [],
        |row| row.get(0),
    )
    .or_else(|_| {
        conn.execute(
            "INSERT OR IGNORE INTO event_cursor (id, last_block) VALUES (1, 0)",
            [],
        )?;
        Ok(0)
    })
}

pub fn update_last_block(conn: &Connection, block: i64) -> SqliteResult<()> {
    conn.execute(
        "INSERT INTO event_cursor (id, last_block, updated_at) VALUES (1, ?1, datetime('now'))
         ON CONFLICT(id) DO UPDATE SET last_block = ?1, updated_at = datetime('now')",
        params![block],
    )?;
    Ok(())
}

// --- Vault operations (RFC-OGB-001 §4.3) ---

pub fn insert_vault(
    conn: &Connection,
    vault_id: &str,
    vault_address: &str,
    group_public_key: &[u8],
) -> SqliteResult<()> {
    conn.execute(
        "INSERT INTO vaults (vault_id, vault_address, group_public_key) VALUES (?1, ?2, ?3)",
        params![vault_id, vault_address, group_public_key],
    )?;
    Ok(())
}

pub fn get_vault(conn: &Connection, vault_id: &str) -> SqliteResult<(String, Vec<u8>, String)> {
    conn.query_row(
        "SELECT vault_address, group_public_key, status FROM vaults WHERE vault_id = ?1",
        params![vault_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )
}

/// Get the current vault state string.
pub fn get_vault_state(conn: &Connection, vault_id: &str) -> SqliteResult<String> {
    conn.query_row(
        "SELECT status FROM vaults WHERE vault_id = ?1",
        params![vault_id],
        |row| row.get(0),
    )
}

/// Transition vault state with audit logging (RFC §9.1).
///
/// Records the transition in `vault_state_transitions` for audit trail.
pub fn update_vault_state(
    conn: &Connection,
    vault_id: &str,
    new_state: &str,
    reason: Option<&str>,
    block_height: Option<i64>,
) -> SqliteResult<()> {
    let old_state: String = conn.query_row(
        "SELECT status FROM vaults WHERE vault_id = ?1",
        params![vault_id],
        |row| row.get(0),
    )?;

    conn.execute(
        "UPDATE vaults SET status = ?1, updated_at = datetime('now') WHERE vault_id = ?2",
        params![new_state, vault_id],
    )?;

    conn.execute(
        "INSERT INTO vault_state_transitions (vault_id, from_state, to_state, reason, block_height) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![vault_id, old_state, new_state, reason, block_height],
    )?;

    Ok(())
}

/// Update vault balance (from chain sync).
pub fn update_vault_balance(
    conn: &Connection,
    vault_id: &str,
    balance_zat: i64,
) -> SqliteResult<()> {
    conn.execute(
        "UPDATE vaults SET balance_zat = ?1, updated_at = datetime('now') WHERE vault_id = ?2",
        params![balance_zat, vault_id],
    )?;
    Ok(())
}

/// Get vault state transition audit log.
pub fn get_vault_transitions(
    conn: &Connection,
    vault_id: &str,
) -> SqliteResult<Vec<(String, String, String, Option<String>, Option<i64>)>> {
    let mut stmt = conn.prepare(
        "SELECT from_state, to_state, created_at, reason, block_height FROM vault_state_transitions WHERE vault_id = ?1 ORDER BY id",
    )?;
    let rows = stmt.query_map(params![vault_id], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
    })?;
    rows.collect()
}

/// Record a vault revocation (RFC §9.2).
pub fn insert_revocation(
    conn: &Connection,
    vault_id: &str,
    owner_signature: &[u8],
    reason: Option<&str>,
) -> SqliteResult<()> {
    conn.execute(
        "INSERT INTO vault_revocations (vault_id, owner_signature, reason) VALUES (?1, ?2, ?3)",
        params![vault_id, owner_signature, reason],
    )?;
    Ok(())
}

/// Get full vault info including lifecycle fields.
pub fn get_vault_full(
    conn: &Connection,
    vault_id: &str,
) -> SqliteResult<(String, String, i64, i64, i64, i64, i64)> {
    conn.query_row(
        "SELECT vault_address, status, balance_zat, last_scanned_height, notes_received, authorized_spends, unauthorized_spends FROM vaults WHERE vault_id = ?1",
        params![vault_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().expect("in-memory db");
        init_db(&conn).expect("init_db");
        conn
    }

    #[test]
    fn test_create_tables() {
        let conn = test_db();

        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<SqliteResult<_>>()
            .unwrap();

        assert!(tables.contains(&"positions".to_string()));
        assert!(tables.contains(&"notes".to_string()));
        assert!(tables.contains(&"loans".to_string()));
        assert!(tables.contains(&"withdrawals".to_string()));
        assert!(tables.contains(&"event_cursor".to_string()));
        assert!(tables.contains(&"vaults".to_string()));
        assert!(tables.contains(&"vault_state_transitions".to_string()));
        assert!(tables.contains(&"vault_revocations".to_string()));
    }

    #[test]
    fn test_insert_position() {
        let conn = test_db();

        insert_position(&conn, "pos-1", "zs1abc", &[1; 96], &[2; 64], &[3; 64]).unwrap();

        let (balance, borrowed, notes) = get_position_balance(&conn, "pos-1").unwrap();
        assert_eq!(balance, 0, "initial balance must be 0");
        assert_eq!(borrowed, 0, "initial borrowed must be 0");
        assert_eq!(notes, 0, "initial note_count must be 0");
    }

    #[test]
    fn test_insert_note() {
        let conn = test_db();

        insert_position(&conn, "pos-1", "zs1abc", &[1; 96], &[2; 64], &[3; 64]).unwrap();
        let note_id =
            insert_note(&conn, "pos-1", "txid-abc", 1_000_000, Some(&[4; 32]), None).unwrap();

        assert!(note_id > 0, "note ID must be positive");

        let notes = get_notes_for_position(&conn, "pos-1").unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].1, "txid-abc");
        assert_eq!(notes[0].2, 1_000_000);
        assert!(!notes[0].3, "note must not be spent");
    }

    #[test]
    fn test_insert_loan() {
        let conn = test_db();

        insert_position(&conn, "pos-1", "zs1abc", &[1; 96], &[2; 64], &[3; 64]).unwrap();
        let loan_id =
            insert_loan(&conn, "pos-1", 100_000_000, "0xAlice", Some("0xtxhash")).unwrap();

        assert!(loan_id > 0, "loan ID must be positive");

        let status: String = conn
            .query_row(
                "SELECT status FROM loans WHERE id = ?1",
                params![loan_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(status, "active", "default loan status must be 'active'");
    }

    #[test]
    fn test_update_balance() {
        let conn = test_db();

        insert_position(&conn, "pos-1", "zs1abc", &[1; 96], &[2; 64], &[3; 64]).unwrap();
        update_balance(&conn, "pos-1", 5_000_000).unwrap();

        let (balance, _, _) = get_position_balance(&conn, "pos-1").unwrap();
        assert_eq!(balance, 5_000_000, "balance must be updated");
    }

    #[test]
    fn test_insert_withdrawal() {
        let conn = test_db();

        insert_position(&conn, "pos-1", "zs1abc", &[1; 96], &[2; 64], &[3; 64]).unwrap();
        let loan_id = insert_loan(&conn, "pos-1", 100_000_000, "0xAlice", Some("0xtx")).unwrap();

        let wd_id = insert_withdrawal(
            &conn,
            "pos-1",
            loan_id,
            "0xavax_hash_1",
            12345,
            &[5; 32],
            &[6; 32],
            1_000_000,
            "zs1recipient",
        )
        .unwrap();

        assert!(wd_id > 0, "withdrawal ID must be positive");

        let status: String = conn
            .query_row(
                "SELECT zec_status FROM withdrawals WHERE id = ?1",
                params![wd_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(status, "pending", "default zec_status must be 'pending'");
    }

    #[test]
    fn test_withdrawal_idempotency() {
        let conn = test_db();

        insert_position(&conn, "pos-1", "zs1abc", &[1; 96], &[2; 64], &[3; 64]).unwrap();
        let loan_id = insert_loan(&conn, "pos-1", 100_000_000, "0xAlice", Some("0xtx")).unwrap();

        // First insertion succeeds
        insert_withdrawal(
            &conn,
            "pos-1",
            loan_id,
            "0xsame_hash",
            100,
            &[7; 32],
            &[8; 32],
            500_000,
            "zs1dest",
        )
        .unwrap();

        // Duplicate avax_tx_hash must fail (UNIQUE constraint)
        let result = insert_withdrawal(
            &conn,
            "pos-1",
            loan_id,
            "0xsame_hash",
            101,
            &[9; 32],
            &[10; 32],
            600_000,
            "zs1dest2",
        );
        assert!(result.is_err(), "duplicate avax_tx_hash must be rejected");
    }

    #[test]
    fn test_event_cursor_singleton() {
        let conn = test_db();

        // First call initializes to 0
        let block = get_last_block(&conn).unwrap();
        assert_eq!(block, 0, "initial last_block must be 0");

        // Update to 100
        update_last_block(&conn, 100).unwrap();
        let block = get_last_block(&conn).unwrap();
        assert_eq!(block, 100, "last_block must be updated to 100");

        // Update to 200
        update_last_block(&conn, 200).unwrap();
        let block = get_last_block(&conn).unwrap();
        assert_eq!(block, 200, "last_block must be updated to 200");

        // CHECK constraint: only id=1 allowed
        let result = conn.execute(
            "INSERT INTO event_cursor (id, last_block, updated_at) VALUES (2, 0, datetime('now'))",
            [],
        );
        assert!(result.is_err(), "event_cursor must only allow id=1");
    }

    #[test]
    fn test_note_requires_valid_position() {
        let conn = test_db();

        // Insert note without a valid position → FK violation
        let result = insert_note(&conn, "nonexistent", "txid-x", 100, None, None);
        assert!(
            result.is_err(),
            "note must reference a valid position (FK constraint)"
        );
    }

    // --- Vault lifecycle tests (RFC §9.1) ---

    #[test]
    fn test_vault_default_inactive() {
        let conn = test_db();
        insert_vault(&conn, "v1", "zs1vault", &[1u8; 32]).unwrap();

        let state = get_vault_state(&conn, "v1").unwrap();
        assert_eq!(state, "inactive", "new vault must default to inactive");
    }

    #[test]
    fn test_vault_state_transition_with_audit() {
        let conn = test_db();
        insert_vault(&conn, "v1", "zs1vault", &[1u8; 32]).unwrap();

        // inactive -> active
        update_vault_state(&conn, "v1", "active", Some("deposit confirmed"), Some(100)).unwrap();
        assert_eq!(get_vault_state(&conn, "v1").unwrap(), "active");

        // active -> delegated
        update_vault_state(&conn, "v1", "delegated", Some("tickets issued"), None).unwrap();
        assert_eq!(get_vault_state(&conn, "v1").unwrap(), "delegated");

        // Check audit log
        let transitions = get_vault_transitions(&conn, "v1").unwrap();
        assert_eq!(transitions.len(), 2);
        assert_eq!(transitions[0].0, "inactive"); // from
        assert_eq!(transitions[0].1, "active");   // to
        assert_eq!(transitions[0].3, Some("deposit confirmed".to_string()));
        assert_eq!(transitions[0].4, Some(100));
        assert_eq!(transitions[1].0, "active");
        assert_eq!(transitions[1].1, "delegated");
    }

    #[test]
    fn test_vault_balance_update() {
        let conn = test_db();
        insert_vault(&conn, "v1", "zs1vault", &[1u8; 32]).unwrap();

        update_vault_balance(&conn, "v1", 5_000_000).unwrap();

        let (_, _, balance, _, _, _, _) = get_vault_full(&conn, "v1").unwrap();
        assert_eq!(balance, 5_000_000);
    }

    #[test]
    fn test_vault_full_info() {
        let conn = test_db();
        insert_vault(&conn, "v1", "zs1vault", &[1u8; 32]).unwrap();

        let (addr, status, balance, height, notes, auth, unauth) =
            get_vault_full(&conn, "v1").unwrap();
        assert_eq!(addr, "zs1vault");
        assert_eq!(status, "inactive");
        assert_eq!(balance, 0);
        assert_eq!(height, 0);
        assert_eq!(notes, 0);
        assert_eq!(auth, 0);
        assert_eq!(unauth, 0);
    }

    #[test]
    fn test_vault_revocation_record() {
        let conn = test_db();
        insert_vault(&conn, "v1", "zs1vault", &[1u8; 32]).unwrap();

        insert_revocation(&conn, "v1", &[0xAA; 64], Some("owner revoked")).unwrap();

        let reason: String = conn
            .query_row(
                "SELECT reason FROM vault_revocations WHERE vault_id = ?1",
                params!["v1"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(reason, "owner revoked");
    }
}

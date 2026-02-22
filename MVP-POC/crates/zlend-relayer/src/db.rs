use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: String,
    pub address: String,
    pub vk: Vec<u8>,
    pub sk_encrypted: Vec<u8>,
    pub ivk: Vec<u8>,
    pub balance_zat: i64,
    pub borrowed_zat: i64,
    pub note_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: i64,
    pub position_id: String,
    pub txid: String,
    pub value_zat: i64,
    pub found_at: String,
    pub spent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Loan {
    pub id: i64,
    pub position_id: String,
    pub amount: i64,
    pub recipient: String,
    pub tx_hash: Option<String>,
    pub status: String,
    pub created_at: String,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn init(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS positions (
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
            );"
        )?;
        Ok(())
    }

    pub fn insert_position(
        &self,
        id: &str,
        address: &str,
        vk: &[u8],
        sk_encrypted: &[u8],
        ivk: &[u8],
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO positions (id, address, vk, sk_encrypted, ivk) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, address, vk, sk_encrypted, ivk],
        )?;
        Ok(())
    }

    pub fn get_position(&self, id: &str) -> Result<Option<Position>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, address, vk, sk_encrypted, ivk, balance_zat, borrowed_zat, note_count, created_at, updated_at
             FROM positions WHERE id = ?1"
        )?;

        let mut rows = stmt.query_map(params![id], |row| {
            Ok(Position {
                id: row.get(0)?,
                address: row.get(1)?,
                vk: row.get(2)?,
                sk_encrypted: row.get(3)?,
                ivk: row.get(4)?,
                balance_zat: row.get(5)?,
                borrowed_zat: row.get(6)?,
                note_count: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?;

        match rows.next() {
            Some(Ok(pos)) => Ok(Some(pos)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    pub fn list_positions(&self) -> Result<Vec<Position>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, address, vk, sk_encrypted, ivk, balance_zat, borrowed_zat, note_count, created_at, updated_at
             FROM positions ORDER BY created_at DESC"
        )?;

        let positions = stmt.query_map([], |row| {
            Ok(Position {
                id: row.get(0)?,
                address: row.get(1)?,
                vk: row.get(2)?,
                sk_encrypted: row.get(3)?,
                ivk: row.get(4)?,
                balance_zat: row.get(5)?,
                borrowed_zat: row.get(6)?,
                note_count: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

        Ok(positions)
    }

    pub fn insert_note(
        &self,
        position_id: &str,
        txid: &str,
        value_zat: i64,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Insert the note
        conn.execute(
            "INSERT INTO notes (position_id, txid, value_zat) VALUES (?1, ?2, ?3)",
            params![position_id, txid, value_zat],
        )?;

        // Update position balance and note count
        conn.execute(
            "UPDATE positions SET
                balance_zat = balance_zat + ?1,
                note_count = note_count + 1,
                updated_at = datetime('now')
             WHERE id = ?2",
            params![value_zat, position_id],
        )?;

        Ok(())
    }

    pub fn check_note_exists(&self, position_id: &str, txid: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM notes WHERE position_id = ?1 AND txid = ?2",
            params![position_id, txid],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn insert_loan(
        &self,
        position_id: &str,
        amount: i64,
        recipient: &str,
        tx_hash: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT INTO loans (position_id, amount, recipient, tx_hash) VALUES (?1, ?2, ?3, ?4)",
            params![position_id, amount, recipient, tx_hash],
        )?;

        // Update position borrowed amount
        conn.execute(
            "UPDATE positions SET
                borrowed_zat = borrowed_zat + ?1,
                updated_at = datetime('now')
             WHERE id = ?2",
            params![amount, position_id],
        )?;

        Ok(())
    }

    pub fn has_active_loan(&self, position_id: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM loans WHERE position_id = ?1 AND status = 'active'",
            params![position_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> Database {
        let db = Database::new(":memory:").unwrap();
        db.init().unwrap();
        db
    }

    #[test]
    fn test_insert_and_get_position() {
        let db = test_db();
        db.insert_position("pos-1", "zs1test", &[1, 2, 3], &[4, 5, 6], &[7, 8, 9])
            .unwrap();

        let pos = db.get_position("pos-1").unwrap().unwrap();
        assert_eq!(pos.id, "pos-1");
        assert_eq!(pos.address, "zs1test");
        assert_eq!(pos.balance_zat, 0);
        assert_eq!(pos.borrowed_zat, 0);
    }

    #[test]
    fn test_insert_note_updates_balance() {
        let db = test_db();
        db.insert_position("pos-1", "zs1test", &[1], &[2], &[3])
            .unwrap();

        db.insert_note("pos-1", "txid-abc", 150_000_000).unwrap();

        let pos = db.get_position("pos-1").unwrap().unwrap();
        assert_eq!(pos.balance_zat, 150_000_000);
        assert_eq!(pos.note_count, 1);
    }

    #[test]
    fn test_insert_loan_updates_borrowed() {
        let db = test_db();
        db.insert_position("pos-1", "zs1test", &[1], &[2], &[3])
            .unwrap();

        db.insert_loan("pos-1", 100_000_000, "0xAlice", Some("0xhash"))
            .unwrap();

        let pos = db.get_position("pos-1").unwrap().unwrap();
        assert_eq!(pos.borrowed_zat, 100_000_000);
        assert!(db.has_active_loan("pos-1").unwrap());
    }

    #[test]
    fn test_check_note_exists() {
        let db = test_db();
        db.insert_position("pos-1", "zs1test", &[1], &[2], &[3])
            .unwrap();

        assert!(!db.check_note_exists("pos-1", "txid-1").unwrap());
        db.insert_note("pos-1", "txid-1", 50_000_000).unwrap();
        assert!(db.check_note_exists("pos-1", "txid-1").unwrap());
    }
}

use anyhow::Result;
use rusqlite::Connection;
use serde::Serialize;
use std::path::Path;

/// Usage history persistence using SQLite.
/// Records device events for audit trail and analytics.
pub struct HistoryDb {
    conn: Connection,
}

#[derive(Debug, Clone, Serialize)]
pub struct HistoryEntry {
    pub id: i64,
    pub timestamp: String,
    pub event_type: String,
    pub bus_id: Option<String>,
    pub device_name: Option<String>,
    pub client_ip: Option<String>,
    pub username: Option<String>,
    pub details: Option<String>,
}

impl HistoryDb {
    /// Open or create the history database (uses same DB file as users).
    pub fn open(path: &str) -> Result<Self> {
        let dir = Path::new(path).parent().unwrap_or(Path::new("."));
        std::fs::create_dir_all(dir)?;
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")?;
        let db = Self { conn };
        db.init()?;
        Ok(db)
    }

    fn init(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL DEFAULT (datetime('now')),
                event_type TEXT NOT NULL,
                bus_id TEXT,
                device_name TEXT,
                client_ip TEXT,
                username TEXT,
                details TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_history_timestamp ON history(timestamp);
            CREATE INDEX IF NOT EXISTS idx_history_event_type ON history(event_type);",
        )?;
        Ok(())
    }

    /// Record an event in history.
    pub fn record(
        &self,
        event_type: &str,
        bus_id: Option<&str>,
        device_name: Option<&str>,
        client_ip: Option<&str>,
        username: Option<&str>,
        details: Option<&str>,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO history (event_type, bus_id, device_name, client_ip, username, details) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![event_type, bus_id, device_name, client_ip, username, details],
        )?;
        Ok(())
    }

    /// Query recent history entries.
    pub fn recent(&self, limit: usize) -> Result<Vec<HistoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, event_type, bus_id, device_name, client_ip, username, details FROM history ORDER BY id DESC LIMIT ?1",
        )?;
        let entries = stmt
            .query_map(rusqlite::params![limit], |row| {
                Ok(HistoryEntry {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    event_type: row.get(2)?,
                    bus_id: row.get(3)?,
                    device_name: row.get(4)?,
                    client_ip: row.get(5)?,
                    username: row.get(6)?,
                    details: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(entries)
    }

    /// Query history filtered by event type.
    pub fn by_type(&self, event_type: &str, limit: usize) -> Result<Vec<HistoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, event_type, bus_id, device_name, client_ip, username, details FROM history WHERE event_type = ?1 ORDER BY id DESC LIMIT ?2",
        )?;
        let entries = stmt
            .query_map(rusqlite::params![event_type, limit], |row| {
                Ok(HistoryEntry {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    event_type: row.get(2)?,
                    bus_id: row.get(3)?,
                    device_name: row.get(4)?,
                    client_ip: row.get(5)?,
                    username: row.get(6)?,
                    details: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(entries)
    }

    /// Purge entries older than N days.
    pub fn purge_older_than(&self, days: u32) -> Result<usize> {
        let deleted = self.conn.execute(
            "DELETE FROM history WHERE timestamp < datetime('now', ?1)",
            rusqlite::params![format!("-{} days", days)],
        )?;
        Ok(deleted)
    }
}

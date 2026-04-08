use anyhow::{Context, Result};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::Path;

use super::password;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String, // "admin", "user", "viewer"
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub password: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUser {
    pub password: Option<String>,
    pub role: Option<String>,
    pub enabled: Option<bool>,
}

pub struct UserDb {
    conn: Connection,
}

impl UserDb {
    /// Open or create the user database.
    pub fn open(path: &str) -> Result<Self> {
        let dir = Path::new(path).parent().unwrap_or(Path::new("."));
        std::fs::create_dir_all(dir)?;
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")?;
        let db = Self { conn };
        db.init()?;
        Ok(db)
    }

    /// Open an in-memory database (for testing).
    pub fn open_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.init()?;
        Ok(db)
    }

    fn init(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                role TEXT NOT NULL DEFAULT 'user',
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )?;

        // Create default admin if no users exist
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
        if count == 0 {
            let hash = password::hash_password("admin")
                .context("Failed to hash default admin password")?;
            self.conn.execute(
                "INSERT INTO users (username, password_hash, role) VALUES (?1, ?2, 'admin')",
                rusqlite::params![&"admin", &hash],
            )?;
            tracing::info!("Created default admin user (password: admin) — change this immediately!");
        }

        Ok(())
    }

    /// Authenticate a user by username and password.
    pub fn authenticate(&self, username: &str, pwd: &str) -> Result<Option<User>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, username, password_hash, role, enabled, created_at FROM users WHERE username = ?1")?;
        let user = stmt.query_row(rusqlite::params![username], |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                password_hash: row.get(2)?,
                role: row.get(3)?,
                enabled: row.get(4)?,
                created_at: row.get(5)?,
            })
        });

        match user {
            Ok(u) => {
                if !u.enabled {
                    return Ok(None);
                }
                if password::verify_password(pwd, &u.password_hash) {
                    Ok(Some(u))
                } else {
                    Ok(None)
                }
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// List all users (without password hashes).
    pub fn list_users(&self) -> Result<Vec<User>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, username, password_hash, role, enabled, created_at FROM users ORDER BY id")?;
        let users = stmt
            .query_map([], |row| {
                Ok(User {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    password_hash: String::new(), // Don't expose
                    role: row.get(3)?,
                    enabled: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(users)
    }

    /// Create a new user.
    pub fn create_user(&self, create: &CreateUser) -> Result<User> {
        let hash = password::hash_password(&create.password)?;
        self.conn.execute(
            "INSERT INTO users (username, password_hash, role) VALUES (?1, ?2, ?3)",
            rusqlite::params![&create.username, &hash, &create.role],
        )?;
        let id = self.conn.last_insert_rowid();
        Ok(User {
            id,
            username: create.username.clone(),
            password_hash: String::new(),
            role: create.role.clone(),
            enabled: true,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Update an existing user.
    pub fn update_user(&self, username: &str, update: &UpdateUser) -> Result<bool> {
        if update.password.is_none() && update.role.is_none() && update.enabled.is_none() {
            return Ok(false);
        }

        let mut updated = false;

        if let Some(ref pwd) = update.password {
            let hash = password::hash_password(pwd)?;
            let changes = self.conn.execute(
                "UPDATE users SET password_hash = ?1 WHERE username = ?2",
                rusqlite::params![&hash, username],
            )?;
            if changes > 0 { updated = true; }
        }
        if let Some(ref role) = update.role {
            let changes = self.conn.execute(
                "UPDATE users SET role = ?1 WHERE username = ?2",
                rusqlite::params![role, username],
            )?;
            if changes > 0 { updated = true; }
        }
        if let Some(enabled) = update.enabled {
            let changes = self.conn.execute(
                "UPDATE users SET enabled = ?1 WHERE username = ?2",
                rusqlite::params![enabled, username],
            )?;
            if changes > 0 { updated = true; }
        }
        Ok(updated)
    }

    /// Delete a user.
    pub fn delete_user(&self, username: &str) -> Result<bool> {
        let changes = self.conn.execute(
            "DELETE FROM users WHERE username = ?1",
            rusqlite::params![username],
        )?;
        Ok(changes > 0)
    }

    /// Get a user by username.
    pub fn get_user(&self, username: &str) -> Result<Option<User>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, username, password_hash, role, enabled, created_at FROM users WHERE username = ?1")?;
        let user = stmt.query_row(rusqlite::params![username], |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                password_hash: String::new(),
                role: row.get(3)?,
                enabled: row.get(4)?,
                created_at: row.get(5)?,
            })
        });

        match user {
            Ok(u) => Ok(Some(u)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

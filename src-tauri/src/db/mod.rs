// LidBridge — Open-Source Desktop Tool for Cleaning and Publishing Projects to GitHub
// Copyright (C) 2026 Lidprex Labs <https://lidprex.onrender.com>
// SPDX-License-Identifier: GPL-3.0-or-later

// =============================================================================
// DEPRECATED — Central DB disabled as of v2.0.0
// This module is kept for reference only. It is NOT called anywhere in the app.
// It will be re-enabled in a future version with a proper backend/middleware.
// Do NOT re-enable without routing through a secure backend.
// =============================================================================
use rusqlite::{Connection, params};
use std::path::PathBuf;

pub mod central;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self, String> {
        let db_path = Self::get_db_path()?;

        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create data directory: {}", e))?;
        }

        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY,
                github_access_token TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        ).map_err(|e| format!("Failed to create sessions table: {}", e))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS repo_history (
                id INTEGER PRIMARY KEY,
                github_id TEXT NOT NULL,
                repo_name TEXT NOT NULL,
                repo_url TEXT NOT NULL,
                owner_type TEXT NOT NULL,
                owner_name TEXT,
                created_at TEXT NOT NULL
            )",
            [],
        ).map_err(|e| format!("Failed to create repo_history table: {}", e))?;

        let _ = conn.execute("ALTER TABLE sessions ADD COLUMN github_id TEXT", []);
        let _ = conn.execute("ALTER TABLE sessions ADD COLUMN email TEXT", []);
        let _ = conn.execute("ALTER TABLE sessions ADD COLUMN name TEXT", []);
        let _ = conn.execute("ALTER TABLE sessions ADD COLUMN avatar_url TEXT", []);
        let _ = conn.execute("ALTER TABLE sessions ADD COLUMN installation_id TEXT", []);

        log::info!("Database initialized at {:?}", db_path);

        Ok(Self { conn })
    }

    fn get_db_path() -> Result<PathBuf, String> {
        let data_dir = dirs::data_local_dir()
            .ok_or("Failed to get local data directory")?;

        Ok(data_dir.join("LidBridge").join("lidbridge.db"))
    }

    pub fn save_session_token(&self, token: &str, github_id: &str, email: &str, name: &str, avatar_url: &str, installation_id: &str) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();

        self.conn.execute("DELETE FROM sessions", [])
            .map_err(|e| format!("Failed to clear sessions: {}", e))?;

        self.conn.execute(
            "INSERT INTO sessions (github_access_token, github_id, email, name, avatar_url, created_at, installation_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![token, github_id, email, name, avatar_url, now, installation_id],
        ).map_err(|e| format!("Failed to save session: {}", e))?;

        log::info!("Session token saved");
        Ok(())
    }

    pub fn get_session_token(&self) -> Result<Option<String>, String> {
        let mut stmt = self.conn
            .prepare("SELECT github_access_token FROM sessions LIMIT 1")
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let result = stmt.query_row([], |row| row.get(0));

        match result {
            Ok(token) => Ok(Some(token)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("Failed to get session: {}", e)),
        }
    }

    pub fn get_user_info(&self) -> Result<Option<(String, String, String, String)>, String> {
        let mut stmt = self.conn
            .prepare("SELECT github_id, email, name, avatar_url FROM sessions LIMIT 1")
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let result = stmt.query_row([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        });

        match result {
            Ok(user) => Ok(Some(user)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("Failed to get user info: {}", e)),
        }
    }

    pub fn clear_session(&self) -> Result<(), String> {
        self.conn.execute("DELETE FROM sessions", [])
            .map_err(|e| format!("Failed to clear session: {}", e))?;

        log::info!("Session cleared");
        Ok(())
    }

    pub fn save_repo_history(&self, github_id: &str, repo_name: &str, repo_url: &str, owner_type: &str, owner_name: &str) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO repo_history (github_id, repo_name, repo_url, owner_type, owner_name, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![github_id, repo_name, repo_url, owner_type, owner_name, now],
        ).map_err(|e| format!("Failed to save repo history: {}", e))?;
        Ok(())
    }

    pub fn get_repo_history(&self) -> Result<Vec<(String, String, String, String, String)>, String> {
        let mut stmt = self.conn
            .prepare("SELECT repo_name, repo_url, owner_type, owner_name, created_at FROM repo_history ORDER BY created_at DESC")
            .map_err(|e| format!("Failed to prepare repo history query: {}", e))?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        }).map_err(|e| format!("Failed to query repo history: {}", e))?;

        let mut history = Vec::new();
        for row in rows {
            history.push(row.map_err(|e| format!("Failed to read repo history row: {}", e))?);
        }

        Ok(history)
    }

    pub fn save_installation_id(&self, installation_id: &str) -> Result<(), String> {
        self.conn.execute(
            "UPDATE sessions SET installation_id = ?1 WHERE id = (SELECT id FROM sessions ORDER BY id DESC LIMIT 1)",
            [installation_id],
        ).map_err(|e| format!("Failed to save installation id: {}", e))?;
        Ok(())
    }

    pub fn get_installation_id(&self) -> Result<Option<String>, String> {
        let _ = self.conn.execute("ALTER TABLE sessions ADD COLUMN installation_id TEXT", []);

        let mut stmt = self.conn
            .prepare("SELECT installation_id FROM sessions LIMIT 1")
            .map_err(|e| format!("Failed to prepare: {}", e))?;

        let result = stmt.query_row([], |row| {
            let value: Option<String> = row.get(0)?;
            Ok(value)
        });

        match result {
            Ok(Some(id)) => Ok(Some(id)),
            Ok(None) => Ok(None),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("Failed to get installation id: {}", e)),
        }
    }
}

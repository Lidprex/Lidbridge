// LidBridge — Open-Source Desktop Tool for Cleaning and Publishing Projects to GitHub
// Copyright (C) 2026 Lidprex Labs <https://lidprex.onrender.com>
// SPDX-License-Identifier: GPL-3.0-or-later

// =============================================================================
// DEPRECATED — Central DB disabled as of v2.0.0
// This module is kept for reference only. It is NOT called anywhere in the app.
// It will be re-enabled in a future version with a proper backend/middleware.
// Do NOT re-enable without routing through a secure backend.
// =============================================================================
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::Row;
use std::time::Duration;

pub struct CentralDb {
    pool: PgPool,
}

#[derive(Debug, Clone)]
pub struct UserStats {
    pub clean_count: i32,
    pub repo_count: i32,
    pub total_size_cleaned: i64,
}

impl CentralDb {
    pub async fn new() -> Result<Self, String> {
        let database_url = option_env!("DATABASE_URL")
            .map(|s| s.to_string())
            .or_else(|| std::env::var("DATABASE_URL").ok())
            .ok_or("DATABASE_URL not found. Central DB disabled.".to_string())?;

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .min_connections(1)
            .max_lifetime(Duration::from_secs(1800))
            .idle_timeout(Duration::from_secs(300))
            .connect(&database_url)
            .await
            .map_err(|e| format!("Failed to connect: {}", e))?;

        Ok(Self { pool })
    }

    pub async fn upsert_user(&self, github_id: &str, username: &str, email: &str, avatar_url: &str) -> Result<i32, String> {
        let row = sqlx::query(
            r#"
            INSERT INTO users (github_id, github_username, email, avatar_url, last_seen)
            VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP)
            ON CONFLICT (github_id)
            DO UPDATE SET
                github_username = EXCLUDED.github_username,
                email = EXCLUDED.email,
                avatar_url = EXCLUDED.avatar_url,
                last_seen = CURRENT_TIMESTAMP
            RETURNING id
            "#
        )
        .bind(github_id)
        .bind(username)
        .bind(email)
        .bind(avatar_url)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to upsert user: {}", e))?;

        let user_id: i32 = row.try_get::<i32, _>("id").unwrap_or_else(|_| {
            row.try_get::<i64, _>("id").unwrap_or(0) as i32
        });

        let _ = sqlx::query(
            r#"
            INSERT INTO stats (user_id, clean_count, repo_count, total_size_cleaned)
            VALUES ($1, 0, 0, 0)
            ON CONFLICT (user_id) DO NOTHING
            "#
        )
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create stats: {}", e))?;

        Ok(user_id)
    }

    pub async fn log_clean(&self, github_id: &str, size_cleaned: u64, repo_name: &str) -> Result<(), String> {
        let user_row = sqlx::query("SELECT id FROM users WHERE github_id = $1")
            .bind(github_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| format!("Failed to get user: {}", e))?;

        let user_id = match user_row {
            Some(row) => row.try_get::<i32, _>("id").unwrap_or_else(|_| {
                row.try_get::<i64, _>("id").unwrap_or(0) as i32
            }),
            None => return Err("User not found".to_string()),
        };

        let _ = sqlx::query(
            r#"
            UPDATE stats
            SET
                clean_count = clean_count + 1,
                total_size_cleaned = total_size_cleaned + $2,
                last_clean = CURRENT_TIMESTAMP,
                updated_at = CURRENT_TIMESTAMP
            WHERE user_id = $1
            "#
        )
        .bind(user_id)
        .bind(size_cleaned as i64)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update stats: {}", e))?;

        let _ = sqlx::query(
            r#"
            INSERT INTO activity_log (user_id, action_type, repo_name, size_cleaned, status)
            VALUES ($1, 'clean', $2, $3, 'success')
            "#
        )
        .bind(user_id)
        .bind(repo_name)
        .bind(size_cleaned as i64)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to log activity: {}", e))?;

        Ok(())
    }

    pub async fn log_push(&self, github_id: &str, repo_name: &str) -> Result<(), String> {
        let user_row = sqlx::query("SELECT id FROM users WHERE github_id = $1")
            .bind(github_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| format!("Failed to get user: {}", e))?;

        let user_id = match user_row {
            Some(row) => row.try_get::<i32, _>("id").unwrap_or_else(|_| {
                row.try_get::<i64, _>("id").unwrap_or(0) as i32
            }),
            None => return Err("User not found".to_string()),
        };

        let _ = sqlx::query(
            r#"
            UPDATE stats
            SET
                repo_count = repo_count + 1,
                last_push = CURRENT_TIMESTAMP,
                updated_at = CURRENT_TIMESTAMP
            WHERE user_id = $1
            "#
        )
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update stats: {}", e))?;

        let _ = sqlx::query(
            r#"
            INSERT INTO activity_log (user_id, action_type, repo_name, status)
            VALUES ($1, 'push', $2, 'success')
            "#
        )
        .bind(user_id)
        .bind(repo_name)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to log activity: {}", e))?;

        Ok(())
    }

    pub async fn get_global_stats(&self) -> Result<serde_json::Value, String> {
        let total_users: (i64,) = sqlx::query_as("SELECT COUNT(*) as count FROM users")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("Failed: {}", e))?;

        let total_cleans: (Option<i64>,) = sqlx::query_as("SELECT SUM(clean_count) as total FROM stats")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("Failed: {}", e))?;

        let total_repos: (Option<i64>,) = sqlx::query_as("SELECT SUM(repo_count) as total FROM stats")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("Failed: {}", e))?;

        let total_size: (Option<i64>,) = sqlx::query_as("SELECT SUM(total_size_cleaned) as total FROM stats")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("Failed: {}", e))?;

        Ok(serde_json::json!({
            "total_users": total_users.0,
            "total_cleans": total_cleans.0.unwrap_or(0),
            "total_repos": total_repos.0.unwrap_or(0),
            "total_size_mb": (total_size.0.unwrap_or(0)) / 1024 / 1024,
        }))
    }

    pub async fn update_global_stats(
        &self,
        lines_scanned: i64,
        junk_mb: f64,
        time_ms: i32,
        _ratio: f64,
        secrets: i32,
        vulns: i32,
        success: bool,
        bandwidth_mb: f64,
    ) -> Result<(), String> {
        let success_int = if success { 1 } else { 0 };
        let failed_int = if success { 0 } else { 1 };

        sqlx::query(
            r#"
            UPDATE global_stats SET
                total_lines_scanned = total_lines_scanned + $1,
                total_junk_removed_mb = total_junk_removed_mb + $2,
                total_secrets_blocked = total_secrets_blocked + $3,
                total_vulnerabilities_blocked = total_vulnerabilities_blocked + $4,
                total_bandwidth_saved_mb = total_bandwidth_saved_mb + $5,
                total_successful_pushes = total_successful_pushes + $6,
                total_failed_pushes = total_failed_pushes + $7,
                avg_execution_time_ms = (avg_execution_time_ms + $8) / 2,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = 1
            "#
        )
        .bind(lines_scanned)
        .bind(junk_mb)
        .bind(secrets)
        .bind(vulns)
        .bind(bandwidth_mb)
        .bind(success_int)
        .bind(failed_int)
        .bind(time_ms as i64)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update global stats: {}", e))?;

        Ok(())
    }

    pub async fn get_global_stats_dashboard(&self) -> Result<serde_json::Value, String> {
        let row = sqlx::query("SELECT * FROM global_stats WHERE id = 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("Failed to get global stats: {}", e))?;

        Ok(serde_json::json!({
            "total_lines_scanned": row.get::<i64, _>("total_lines_scanned"),
            "total_junk_removed_mb": row.get::<f64, _>("total_junk_removed_mb"),
            "total_secrets_blocked": row.get::<i32, _>("total_secrets_blocked"),
            "total_vulnerabilities_blocked": row.get::<i32, _>("total_vulnerabilities_blocked"),
            "total_bandwidth_saved_mb": row.get::<f64, _>("total_bandwidth_saved_mb"),
            "total_successful_pushes": row.get::<i32, _>("total_successful_pushes"),
            "total_failed_pushes": row.get::<i32, _>("total_failed_pushes"),
            "avg_execution_time_ms": row.get::<i32, _>("avg_execution_time_ms"),
        }))
    }

    pub async fn log_repo_creation(&self, github_id: &str, repo_name: &str) -> Result<(), String> {
        let user_row = sqlx::query("SELECT id FROM users WHERE github_id = $1")
            .bind(github_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| format!("Failed to get user: {}", e))?;

        let user_id = match user_row {
            Some(row) => row.try_get::<i32, _>("id").unwrap_or_else(|_| {
                row.try_get::<i64, _>("id").unwrap_or(0) as i32
            }),
            None => {
                log::warn!("User {} not found in central db", github_id);
                return Err(format!("User {} not found", github_id));
            }
        };

        let _ = sqlx::query(
            r#"
            UPDATE stats
            SET
                repo_count = repo_count + 1,
                last_push = CURRENT_TIMESTAMP,
                updated_at = CURRENT_TIMESTAMP
            WHERE user_id = $1
            "#
        )
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update stats: {}", e))?;

        let _ = sqlx::query(
            r#"
            INSERT INTO activity_log (user_id, action_type, repo_name, status)
            VALUES ($1, 'push', $2, 'success')
            "#
        )
        .bind(user_id)
        .bind(repo_name)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to log activity: {}", e))?;

        log::info!("Logged repo creation for user {}: {}", github_id, repo_name);
        Ok(())
    }
}

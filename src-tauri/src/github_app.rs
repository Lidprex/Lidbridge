// LidBridge — Open-Source Desktop Tool for Cleaning and Publishing Projects to GitHub
// Copyright (C) 2026 Lidprex Labs <https://lidprex.onrender.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use reqwest::Client;

pub const APP_ID: &str = "3522405";

/// Loads the GitHub App private key at runtime.
///
/// The key is a secret and must NOT be committed, so it is read from the
/// `GITHUB_APP_PRIVATE_KEY` environment variable, or from the optional
/// `src/keys/private-key.pem` file next to the crate. Missing key simply
/// disables the (optional) GitHub-App org-push path — it never aborts the app.
fn load_private_key() -> Result<String, String> {
    if let Ok(key) = std::env::var("GITHUB_APP_PRIVATE_KEY") {
        if !key.trim().is_empty() {
            return Ok(key);
        }
    }

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/keys/private-key.pem");
    match std::fs::read_to_string(&path) {
        Ok(key) if !key.trim().is_empty() => Ok(key),
        _ => Err(
            "GitHub App private key not configured. Set GITHUB_APP_PRIVATE_KEY or add \
             src/keys/private-key.pem to enable GitHub-App organization pushes."
                .to_string(),
        ),
    }
}

#[derive(Debug, Serialize)]
struct JWTClaims {
    iat: u64,
    exp: u64,
    iss: String,
}

#[allow(dead_code)]
pub async fn get_installation_token(installation_id: &str) -> Result<String, String> {
    let private_key = load_private_key()?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("System clock error: {}", e))?
        .as_secs();

    let claims = JWTClaims {
        iat: now,
        exp: now + (10 * 60),
        iss: APP_ID.to_string(),
    };

    let encoding_key = EncodingKey::from_rsa_pem(private_key.as_bytes())
        .map_err(|e| format!("Invalid GitHub App private key: {}", e))?;

    let token = encode(&Header::new(Algorithm::RS256), &claims, &encoding_key)
        .map_err(|e| format!("Failed to sign JWT: {}", e))?;

    let client = Client::new();
    let response = client
        .post(&format!("https://api.github.com/app/installations/{}/access_tokens", installation_id))
        .bearer_auth(token)
        .header("User-Agent", "LidBridge")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| format!("Failed to get installation token: {}", e))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("GitHub API error: {}", error_text));
    }

    let json: serde_json::Value = response.json().await.map_err(|e| format!("Failed to parse response: {}", e))?;
    json["token"].as_str().map(|s| s.to_string()).ok_or("No token in response".to_string())
}

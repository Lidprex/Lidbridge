use serde::{Deserialize, Serialize};
use std::sync::Mutex;

fn env_or(key: &str, fallback: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| fallback.to_string())
}

pub fn github_client_id() -> String {
    env_or("GITHUB_CLIENT_ID", "")
}

pub fn github_client_secret() -> String {
    env_or("GITHUB_CLIENT_SECRET", "")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub id: i64,
    pub login: String,
    pub name: String,
    pub email: String,
    pub avatar_url: String,
}

pub struct AuthManager {
    csrf_token: Mutex<Option<String>>,
}

impl AuthManager {
    pub fn new() -> Self {
        Self {
            csrf_token: Mutex::new(None),
        }
    }

    pub fn build_auth_url(&self, state: &str) -> String {
        let scope = "repo%20write:org%20read:user";
        let client_id = github_client_id();
        format!(
            "https://github.com/login/oauth/authorize?client_id={}&redirect_uri=http://localhost:2026/callback&scope={}&state={}",
            client_id, scope, state
        )
    }

    pub async fn exchange_code_for_token(&self, code: &str) -> Result<String, String> {
        let client = reqwest::Client::new();
        let cid = github_client_id();
        let csecret = github_client_secret();
        let params = [
            ("client_id", cid.as_str()),
            ("client_secret", csecret.as_str()),
            ("code", code),
            ("redirect_uri", "http://localhost:2026/callback"),
        ];

        let response = client
            .post("https://github.com/login/oauth/access_token")
            .form(&params)
            .header("Accept", "application/json")
            .header("User-Agent", "LidBridge")
            .send()
            .await
            .map_err(|e| format!("Failed to exchange code: {}", e))?;

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse token response: {}", e))?;

        if let Some(token) = json["access_token"].as_str() {
            Ok(token.to_string())
        } else {
            let error = json["error_description"].as_str()
                .or_else(|| json["error"].as_str())
                .unwrap_or("Unknown error");
            Err(format!("Token exchange failed: {}", error))
        }
    }

    pub async fn get_github_user(&self, token: &str) -> Result<(String, String, String, String), String> {
        let client = reqwest::Client::new();

        let response = client
            .get("https://api.github.com/user")
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "LidBridge")
            .send()
            .await
            .map_err(|e| format!("Failed to get user: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("GitHub API error: {}", response.status()));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let github_id = json["id"].as_i64().unwrap_or(0).to_string();
        let email = json["email"].as_str().unwrap_or("").to_string();
        let name = json["login"].as_str().unwrap_or("").to_string();
        let avatar_url = json["avatar_url"].as_str().unwrap_or("").to_string();

        Ok((github_id, email, name, avatar_url))
    }

    pub async fn download_avatar(&self, avatar_url: &str) -> Result<String, String> {
        if avatar_url.is_empty() {
            return Ok(String::new());
        }

        let data_dir = dirs::data_local_dir()
            .ok_or("Failed to get local data directory")?;
        let avatar_dir = data_dir.join("LidBridge").join("avatars");
        std::fs::create_dir_all(&avatar_dir)
            .map_err(|e| format!("Failed to create avatar dir: {}", e))?;

        let avatar_path = avatar_dir.join("avatar.png");
        if avatar_path.exists() {
            return Ok(avatar_path.to_string_lossy().to_string());
        }

        let client = reqwest::Client::new();
        let response = client
            .get(avatar_url)
            .header("User-Agent", "LidBridge")
            .send()
            .await
            .map_err(|e| format!("Failed to download avatar: {}", e))?;

        if !response.status().is_success() {
            return Ok(avatar_url.to_string());
        }

        let bytes = response.bytes().await
            .map_err(|e| format!("Failed to read avatar bytes: {}", e))?;

        std::fs::write(&avatar_path, &bytes)
            .map_err(|e| format!("Failed to save avatar: {}", e))?;

        Ok(avatar_path.to_string_lossy().to_string())
    }
}

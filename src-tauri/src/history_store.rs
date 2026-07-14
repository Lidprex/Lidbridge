use std::fs;
use std::path::PathBuf;

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use rand::RngCore;

const APP_ID: &str = "lidbridge-v2-local-state";
const SESSION_FILE: &str = "session-state.bin";
const HISTORY_FILE: &str = "repo-history.bin";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub access_token: String,
    pub github_id: String,
    pub email: String,
    pub name: String,
    pub avatar_url: String,
    pub installation_id: String,
    pub expires_at: String,
}

impl SessionState {
    pub fn is_expired(&self) -> bool {
        DateTime::parse_from_rfc3339(&self.expires_at)
            .map(|ts| Utc::now() >= ts.with_timezone(&Utc))
            .unwrap_or(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoHistoryEntry {
    pub repo_name: String,
    pub repo_url: String,
    pub owner_type: String,
    pub owner_name: String,
    pub created_at: String,
}

pub struct HistoryStore {
    base_dir: PathBuf,
}

impl HistoryStore {
    pub fn new() -> Result<Self, String> {
        Self::with_base_dir(Self::base_dir()?)
    }

    fn with_base_dir(base_dir: impl Into<PathBuf>) -> Result<Self, String> {
        let base_dir = base_dir.into();
        fs::create_dir_all(&base_dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;

        Ok(Self { base_dir })
    }

    fn base_dir() -> Result<PathBuf, String> {
        let data_dir = dirs::data_local_dir().ok_or("Failed to get local data directory")?;
        Ok(data_dir.join("LidBridge"))
    }

    fn session_path(&self) -> PathBuf {
        self.base_dir.join(SESSION_FILE)
    }

    fn history_path(&self) -> PathBuf {
        self.base_dir.join(HISTORY_FILE)
    }

    fn derive_key() -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(APP_ID.as_bytes());
        let out = hasher.finalize();
        let mut key = [0u8; 32];
        key.copy_from_slice(&out[..32]);
        key
    }

    fn encrypt_json<T: Serialize>(value: &T) -> Result<String, String> {
        let json = serde_json::to_vec(value).map_err(|e| format!("Failed to serialize state: {}", e))?;
        let key = Aes256Gcm::new_from_slice(&Self::derive_key()).map_err(|_| "Invalid encryption key".to_string())?;
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = key
            .encrypt(nonce, json.as_ref())
            .map_err(|e| format!("Failed to encrypt state: {}", e))?;
        let mut payload = nonce_bytes.to_vec();
        payload.extend(ciphertext);
        Ok(STANDARD.encode(payload))
    }

    fn decrypt_json<T: for<'de> Deserialize<'de>>(payload: &str) -> Result<T, String> {
        let bytes = STANDARD.decode(payload).map_err(|e| format!("Failed to decode payload: {}", e))?;
        if bytes.len() <= 12 { return Err("Encrypted state is invalid".to_string()); }
        let key = Aes256Gcm::new_from_slice(&Self::derive_key()).map_err(|_| "Invalid encryption key".to_string())?;
        let nonce = Nonce::from_slice(&bytes[..12]);
        let plaintext = key
            .decrypt(nonce, &bytes[12..])
            .map_err(|e| format!("Failed to decrypt state: {}", e))?;
        serde_json::from_slice(&plaintext).map_err(|e| format!("Failed to deserialize state: {}", e))
    }

    pub fn save_session(
        &self,
        access_token: &str,
        github_id: &str,
        email: &str,
        name: &str,
        avatar_url: &str,
        installation_id: &str,
    ) -> Result<(), String> {
        let expires_at = (Utc::now() + Duration::days(7)).to_rfc3339();
        let state = SessionState {
            access_token: access_token.to_string(),
            github_id: github_id.to_string(),
            email: email.to_string(),
            name: name.to_string(),
            avatar_url: avatar_url.to_string(),
            installation_id: installation_id.to_string(),
            expires_at: expires_at.clone(),
        };

        let payload = Self::encrypt_json(&state)?;
        fs::write(self.session_path(), payload).map_err(|e| format!("Failed to write session state: {}", e))?;
        Ok(())
    }

    pub fn load_session(&self) -> Result<Option<SessionState>, String> {
        let path = self.session_path();
        if !path.exists() {
            return Ok(None);
        }

        let payload = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(err) => {
                let _ = self.clear_session();
                return Err(format!("Failed to read session state: {}", err));
            }
        };

        match Self::decrypt_json::<SessionState>(&payload) {
            Ok(state) => {
                if state.is_expired() {
                    let _ = self.clear_session();
                    return Ok(None);
                }
                Ok(Some(state))
            }
            Err(err) => {
                log::warn!("Ignoring corrupt session payload: {}", err);
                let _ = self.clear_session();
                Ok(None)
            }
        }
    }

    pub fn clear_session(&self) -> Result<(), String> {
        if self.session_path().exists() {
            fs::remove_file(self.session_path()).map_err(|e| format!("Failed to clear session state: {}", e))?;
        }
        Ok(())
    }

    pub fn save_repo_history(&self, repo_name: &str, repo_url: &str, owner_type: &str, owner_name: &str) -> Result<(), String> {
        let mut history = self.load_repo_history()?;
        let entry = RepoHistoryEntry {
            repo_name: repo_name.to_string(),
            repo_url: repo_url.to_string(),
            owner_type: owner_type.to_string(),
            owner_name: owner_name.to_string(),
            created_at: Utc::now().to_rfc3339(),
        };
        history.insert(0, entry);
        history.truncate(50);

        let payload = Self::encrypt_json(&history)?;
        fs::write(self.history_path(), payload).map_err(|e| format!("Failed to write history: {}", e))?;
        Ok(())
    }

    pub fn load_repo_history(&self) -> Result<Vec<RepoHistoryEntry>, String> {
        let path = self.history_path();
        if !path.exists() {
            return Ok(Vec::new());
        }

        let payload = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(err) => {
                let _ = fs::remove_file(&path);
                return Err(format!("Failed to read history: {}", err));
            }
        };

        match Self::decrypt_json::<Vec<RepoHistoryEntry>>(&payload) {
            Ok(history) => Ok(history),
            Err(err) => {
                log::warn!("Ignoring corrupt repo history payload: {}", err);
                let _ = fs::remove_file(&path);
                Ok(Vec::new())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_and_history_roundtrip() {
        let base_dir = std::env::temp_dir().join(format!("lidbridge-history-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base_dir);

        let store = HistoryStore::with_base_dir(&base_dir).unwrap();
        store.save_session("token", "123", "user@example.com", "User", "https://example.com/avatar.png", "42").unwrap();
        let session = store.load_session().unwrap().unwrap();
        assert_eq!(session.github_id, "123");
        assert_eq!(session.name, "User");

        store.save_repo_history("demo", "https://github.com/demo", "user", "demo").unwrap();
        let history = store.load_repo_history().unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].repo_name, "demo");

        let _ = fs::remove_dir_all(base_dir);
    }

    #[test]
    fn corrupt_payloads_are_ignored_gracefully() {
        let base_dir = std::env::temp_dir().join(format!("lidbridge-history-corrupt-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base_dir);

        let store = HistoryStore::with_base_dir(&base_dir).unwrap();
        fs::write(store.session_path(), "not-a-valid-payload").unwrap();
        fs::write(store.history_path(), "also-not-valid").unwrap();

        assert!(store.load_session().unwrap().is_none());
        assert!(store.load_repo_history().unwrap().is_empty());

        let _ = fs::remove_dir_all(base_dir);
    }
}

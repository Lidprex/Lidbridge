use std::path::Path;
use std::fs;
use crate::RepoConfig;
use base64::Engine;
use walkdir::WalkDir;

pub struct GitPusher {
    token: String,
}

impl GitPusher {
    pub fn new(token: &str) -> Self {
        Self {
            token: token.to_string(),
        }
    }

    pub async fn create_and_push_with_owner(
        &self,
        project_path: &str,
        config: RepoConfig,
        owner_type: &str,
        owner_name: &str,
        github_username: &str,
        github_email: &str,
    ) -> Result<String, String> {
        let path = Path::new(project_path);
        if !path.exists() {
            return Err("Project path does not exist".to_string());
        }

        let (_clone_url, html_url) = self.create_github_repo_with_owner(&config, owner_type, owner_name).await?;

        let owner = if owner_type == "org" && !owner_name.is_empty() {
            owner_name.to_string()
        } else {
            self.get_github_username().await?
        };

        let mut username = github_username.trim().to_string();
        if username.is_empty() {
            username = "LidBridge User".to_string();
        }

        let email = if github_email.trim().is_empty() {
            "lidbridge@local".to_string()
        } else {
            github_email.to_string()
        };

        self.upload_directory_via_contents_api(path, &owner, &config.name, &username, &email).await?;

        Ok(html_url)
    }

    async fn get_github_username(&self) -> Result<String, String> {
        let client = reqwest::Client::new();
        let resp = client
            .get("https://api.github.com/user")
            .bearer_auth(&self.token)
            .header("User-Agent", "LidBridge")
            .send()
            .await
            .map_err(|e| format!("Failed to get user info: {}", e))?;

        let json: serde_json::Value = resp.json().await.map_err(|e| format!("Failed to parse user info: {}", e))?;
        json["login"].as_str()
            .map(|s| s.to_string())
            .ok_or("Could not determine GitHub username".to_string())
    }

    async fn create_github_repo_with_owner(&self, config: &RepoConfig, owner_type: &str, owner_name: &str) -> Result<(String, String), String> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Failed to build client: {}", e))?;

        let url = if owner_type == "org" && !owner_name.is_empty() {
            format!("https://api.github.com/orgs/{}/repos", owner_name)
        } else {
            "https://api.github.com/user/repos".to_string()
        };

        let body = serde_json::json!({
            "name": config.name,
            "description": config.description,
            "private": config.is_private,
            "auto_init": true,
            "has_issues": true,
            "has_wiki": true,
            "has_downloads": true
        });

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("User-Agent", "LidBridge")
            .header("Accept", "application/vnd.github+json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Failed to create repository: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            if status.as_u16() == 422 {
                return Err(format!("Repository '{}' already exists or name is invalid. Try a different name.", config.name));
            }
            return Err(format!("GitHub API error {}: {}", status, error_text));
        }

        let json: serde_json::Value = response.json().await.map_err(|e| format!("Failed to parse response: {}", e))?;

        let clone_url = json["clone_url"].as_str().ok_or("No clone_url")?.to_string();
        let html_url = json["html_url"].as_str().ok_or("No html_url")?.to_string();

        Ok((clone_url, html_url))
    }

    async fn upload_directory_via_contents_api(
        &self,
        path: &Path,
        owner: &str,
        repo_name: &str,
        author_name: &str,
        author_email: &str,
    ) -> Result<(), String> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| format!("Failed to build client: {}", e))?;

        let mut files_to_upload: Vec<(String, Vec<u8>)> = Vec::new();

        for entry in WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }

            let full_path = entry.path();
            let rel = match full_path.strip_prefix(path) {
                Ok(r) => r,
                Err(_) => continue,
            };

            let rel_str = rel.to_string_lossy().to_string();

            if rel_str == ".git" || rel_str.starts_with(".git\\") || rel_str.starts_with(".git/") {
                continue;
            }

            let content = match fs::read(full_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            if content.len() > 1024 * 1024 {
                continue;
            }

            let api_path = rel_str.replace('\\', "/");
            files_to_upload.push((api_path, content));
        }

        if files_to_upload.is_empty() {
            return Err("No files found to upload".to_string());
        }

        let total = files_to_upload.len();
        let mut uploaded = 0usize;
        let mut failed = 0usize;
        let commit_msg = format!("Initial commit via LidBridge ({} files)", total);

        for (file_path, content) in &files_to_upload {
            let content_str = base64::engine::general_purpose::STANDARD.encode(&content);

            let put_url = format!(
                "https://api.github.com/repos/{}/{}/contents/{}",
                owner, repo_name, file_path
            );

            let body = serde_json::json!({
                "message": &commit_msg,
                "content": content_str,
                "encoding": "base64",
                "author": {
                    "name": author_name,
                    "email": author_email
                },
                "committer": {
                    "name": author_name,
                    "email": author_email
                }
            });

            let mut attempts = 0;
            loop {
                attempts += 1;
                let resp = client
                    .put(&put_url)
                    .header("Authorization", format!("Bearer {}", self.token))
                    .header("User-Agent", "LidBridge")
                    .header("Accept", "application/vnd.github+json")
                    .json(&body)
                    .send()
                    .await;

                match resp {
                    Ok(r) => {
                        let status = r.status();
                        if status.is_success() || status.as_u16() == 201 {
                            uploaded += 1;
                            break;
                        } else if status.as_u16() == 403 || status.as_u16() == 429 {
                            if attempts < 5 {
                                tokio::time::sleep(std::time::Duration::from_secs(attempts * 2)).await;
                                continue;
                            }
                            let err = r.text().await.unwrap_or_default();
                            eprintln!("Rate limited for {}: {}", file_path, err);
                            failed += 1;
                            break;
                        } else if status.as_u16() == 422 {
                            let err = r.text().await.unwrap_or_default();
                            eprintln!("Validation error for {}: {} - {}", file_path, status, err);
                            failed += 1;
                            break;
                        } else {
                            let err = r.text().await.unwrap_or_default();
                            eprintln!("Failed to upload {}: {} - {}", file_path, status, err);
                            failed += 1;
                            break;
                        }
                    }
                    Err(e) => {
                        if attempts < 3 {
                            tokio::time::sleep(std::time::Duration::from_secs(attempts)).await;
                            continue;
                        }
                        eprintln!("Network error uploading {}: {}", file_path, e);
                        failed += 1;
                        break;
                    }
                }
            }
        }

        if uploaded == 0 {
            return Err(format!("Failed to upload any files. {} files failed.", failed));
        }

        Ok(())
    }
}

fn is_binary_file(data: &[u8]) -> bool {
    if data.len() > 1024 * 1024 {
        return true;
    }
    let check_len = data.len().min(8192);
    let mut null_count = 0u32;
    for &byte in &data[..check_len] {
        if byte == 0 {
            null_count += 1;
        }
    }
    null_count > check_len as u32 / 100
}

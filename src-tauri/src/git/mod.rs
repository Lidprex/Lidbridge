// LidBridge — Open-Source Desktop Tool for Cleaning and Publishing Projects to GitHub
// Copyright (C) 2026 Lidprex Labs <https://lidprex.onrender.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::Path;
use crate::RepoConfig;
use git2::{Repository, Signature, IndexAddOption, PushOptions, RemoteCallbacks};

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

        log::info!("Creating GitHub repository: {}", config.name);

        let (clone_url, html_url) = self.create_github_repo_with_owner(&config, owner_type, owner_name).await?;

        let path_buf = path.to_path_buf();
        let clone_url_copy = clone_url.clone();
        let config_copy = config.clone();

        let mut username = github_username.trim().to_string();
        if username.is_empty() {
            username = "LidBridge User".to_string();
        }

        let email = if github_email.trim().is_empty() {
            "lidbridge@local".to_string()
        } else {
            github_email.to_string()
        };

        let token = self.token.clone();

        tokio::task::spawn_blocking(move || -> Result<(), String> {
            let pusher = GitPusher::new(&token);
            pusher.init_git_repo(&path_buf)?;
            pusher.create_initial_commit(&path_buf, &config_copy, &username, &email)?;
            pusher.push_to_github(&path_buf, &clone_url_copy)?;
            Ok(())
        })
        .await
        .map_err(|e| format!("Task failed: {}", e))??;

        Ok(html_url)
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
            "auto_init": false,
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

        log::info!("Repository created: {}", html_url);

        Ok((clone_url, html_url))
    }

    fn init_git_repo(&self, path: &Path) -> Result<(), String> {
        let git_dir = path.join(".git");
        if !git_dir.exists() {
            Repository::init(path).map_err(|e| format!("Failed to initialize repository: {}", e))?;
        }
        Ok(())
    }

    fn create_initial_commit(&self, path: &Path, config: &RepoConfig, username: &str, email: &str) -> Result<(), String> {
        let repo = Repository::open(path).map_err(|e| format!("Failed to open repo: {}", e))?;
        let mut index = repo.index().map_err(|e| format!("Failed to open index: {}", e))?;

        index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
            .map_err(|e| format!("Failed to add files: {}", e))?;
        index.write().map_err(|e| format!("Failed to write index: {}", e))?;

        let oid = index.write_tree().map_err(|e| format!("Failed to write tree: {}", e))?;
        let signature = Signature::now(username, email).map_err(|e| format!("Failed to create signature: {}", e))?;

        let commit_msg = if config.create_readme {
            "Initial commit via LidBridge with README"
        } else {
            "Initial commit via LidBridge"
        };

        let tree = repo.find_tree(oid).map_err(|e| format!("Failed to find tree: {}", e))?;

        match repo.head() {
            Ok(head) => {
                let target = head.target()
                    .ok_or("HEAD has no target commit")?;
                let parent = repo.find_commit(target)
                    .map_err(|e| format!("Failed to find parent commit: {}", e))?;
                repo.commit(Some("HEAD"), &signature, &signature, commit_msg, &tree, &[&parent])
                    .map_err(|e| format!("Failed to create commit: {}", e))?;
            },
            Err(_) => {
                repo.commit(Some("HEAD"), &signature, &signature, commit_msg, &tree, &[])
                    .map_err(|e| format!("Failed to create initial commit: {}", e))?;
            }
        }

        // Ensure branch is named 'main'
        if let Ok(commit) = repo.head().and_then(|h| h.peel_to_commit()) {
            let _ = repo.branch("main", &commit, true);
            let _ = repo.set_head("refs/heads/main");
        }

        Ok(())
    }

    fn push_to_github(&self, path: &Path, repo_url: &str) -> Result<(), String> {
        let repo = Repository::open(path).map_err(|e| format!("Failed to open repo: {}", e))?;

        let mut remote = match repo.find_remote("origin") {
            Ok(r) => r,
            Err(_) => repo.remote("origin", repo_url).map_err(|e| format!("Failed to add remote: {}", e))?
        };

        let mut callbacks = RemoteCallbacks::new();
        let token = self.token.clone();
        callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
            git2::Cred::userpass_plaintext("x-access-token", &token)
        });

        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);

        let refspec_main = "refs/heads/main:refs/heads/main";
        let refspec_master = "refs/heads/master:refs/heads/master";

        if let Err(e) = remote.push(&[refspec_main], Some(&mut push_options)) {
            log::info!("Failed to push main, trying master: {}", e);
            remote.push(&[refspec_master], Some(&mut push_options))
                .map_err(|e2| format!("Failed to push branch: {}", e2))?;
        }

        log::info!("Successfully pushed to GitHub");
        Ok(())
    }
}

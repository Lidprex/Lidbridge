// LidBridge — Open-Source Desktop Tool for Cleaning and Publishing Projects to GitHub
// Copyright (C) 2026 Lidprex Labs <https://lidprex.onrender.com>
// SPDX-License-Identifier: GPL-3.0-or-later
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

#![windows_subsystem = "windows"]

use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use tauri::Emitter;
pub mod auth;
pub mod cleaner;
pub mod git;
pub mod db;
pub mod github_app;

pub struct AppState {
    pub db: Mutex<db::Database>,
    pub auth: Mutex<auth::AuthManager>,
    pub central_db: Mutex<Option<db::central::CentralDb>>,
    pub oauth_callback_tx: Mutex<Option<oneshot::Sender<String>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub github_id: String,
    pub email: String,
    pub name: String,
    pub avatar_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanResult {
    pub success: bool,
    pub cleaned_path: String,
    pub deleted_items: Vec<String>,
    pub warnings: Vec<String>,
    pub total_size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoConfig {
    pub name: String,
    pub description: String,
    pub is_private: bool,
    pub include_images: bool,
    pub create_readme: bool,
}

// ========== OAUTH CALLBACK SERVER ==========

async fn start_oauth_server(app_handle: tauri::AppHandle, port: u16) -> Result<(), String> {
    let addr = format!("127.0.0.1:{}", port);

    log::info!("Starting OAuth callback server on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("Failed to bind to port {}: {}", port, e))?;

    log::info!("OAuth server listening on {}", addr);

    while let Ok((mut stream, _)) = listener.accept().await {
        let app_handle = app_handle.clone();

        tokio::spawn(async move {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};

            let mut buffer = [0u8; 2048];
            if let Ok(n) = stream.read(&mut buffer).await {
                let request = String::from_utf8_lossy(&buffer[..n]);

                if request.contains("GET /callback") || request.contains("GET /?code=") {
                    let code = if let Some(start) = request.find("code=") {
                        let rest = &request[start + 5..];
                        if let Some(end) = rest.find('&') {
                            Some(rest[..end].to_string())
                        } else if let Some(end) = rest.find(' ') {
                            Some(rest[..end].to_string())
                        } else {
                            Some(rest.to_string())
                        }
                    } else {
                        None
                    };

                    let response = if let Some(code) = code {
                        log::info!("Received OAuth code, exchanging for token...");
                        let _ = app_handle.emit("oauth-code-received", &code);
                        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>LidBridge - Authentication Successful</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #0A0A0F 0%, #1a1a2e 100%);
            color: white;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            margin: 0;
        }
        .container { text-align: center; }
        .success { font-size: 48px; margin-bottom: 20px; }
        h1 { color: #00D4FF; margin-bottom: 10px; }
        p { color: #888; }
    </style>
</head>
<body>
    <div class="container">
        <div class="success">✅</div>
        <h1>Authentication Successful!</h1>
        <p>You can close this window and return to LidBridge.</p>
        <script>setTimeout(() => window.close(), 3000);</script>
    </div>
</body>
</html>"#.to_string()
                    } else {
                        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>LidBridge - Error</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #0A0A0F 0%, #1a1a2e 100%);
            color: white;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            margin: 0;
        }
        .container { text-align: center; }
        .error { font-size: 48px; margin-bottom: 20px; }
        h1 { color: #ff4757; margin-bottom: 10px; }
    </style>
</head>
<body>
    <div class="container">
        <div class="error">❌</div>
        <h1>Authentication Failed</h1>
        <p>Please try again.</p>
    </div>
</body>
</html>"#.to_string()
                    };

                    let response_str = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        response.len(),
                        response
                    );
                    let _ = stream.write_all(response_str.as_bytes()).await;
                } else {
                    let response = "HTTP/1.1 302 Found\r\nLocation: /\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                    let _ = stream.write_all(response.as_bytes()).await;
                }
            }
        });
    }

    Ok(())
}

// ========== TAURI COMMANDS ==========

#[tauri::command]
async fn get_session(state: tauri::State<'_, AppState>) -> Result<Option<User>, String> {
    let db = state.db.lock().await;
    match db.get_user_info() {
        Ok(Some((github_id, email, name, avatar_url))) => {
            Ok(Some(User {
                id: 0,
                github_id,
                email,
                name,
                avatar_url,
            }))
        }
        Ok(None) => Ok(None),
        Err(e) => Err(e),
    }
}

#[tauri::command]
async fn get_repo_history(state: tauri::State<'_, AppState>) -> Result<Vec<serde_json::Value>, String> {
    let db = state.db.lock().await;
    let history = db.get_repo_history()?;

    let repos = history.into_iter().map(|(repo_name, repo_url, owner_type, owner_name, created_at)| {
        serde_json::json!({
            "repo_name": repo_name,
            "repo_url": repo_url,
            "owner_type": owner_type,
            "owner_name": owner_name,
            "created_at": created_at,
        })
    }).collect();

    Ok(repos)
}

#[tauri::command]
async fn start_oauth(app_handle: tauri::AppHandle, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let auth_url = {
        let auth = state.auth.lock().await;
        let (auth_url, csrf_token) = auth.get_authorization_url();
        log::info!("OAuth started with CSRF token: {}", csrf_token);
        auth_url
    };

    // Run the callback server on the shared async runtime instead of spawning a
    // std thread with its own nested tokio runtime.
    let app_handle_clone = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = start_oauth_server(app_handle_clone, 2026).await {
            log::error!("OAuth server error: {}", e);
        }
    });

    // Give the callback server a moment to bind before opening the browser.
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    let _ = open::that(&auth_url);

    Ok(auth_url)
}

#[tauri::command]
async fn complete_oauth(code: String, state: tauri::State<'_, AppState>) -> Result<User, String> {
    let auth = state.auth.lock().await;
    let db = state.db.lock().await;

    let access_token = auth.exchange_code_for_token(&code)
        .await
        .map_err(|e| e.to_string())?;

    let (github_id, email, name, avatar_url) = auth.get_github_user(&access_token)
        .await
        .map_err(|e| e.to_string())?;

    let mut installation_id = String::new();
    let client = reqwest::Client::new();

    if let Ok(resp) = client
        .get("https://api.github.com/user/installations")
        .bearer_auth(&access_token)
        .header("User-Agent", "LidBridge")
        .send()
        .await
    {
        if let Ok(json) = resp.json::<serde_json::Value>().await {
            if let Some(installs) = json["installations"].as_array() {
                for inst in installs {
                    if inst["app_id"].as_i64() == Some(3522405) {
                        if let Some(id) = inst["id"].as_i64() {
                            installation_id = id.to_string();
                            break;
                        }
                    }
                }
            }
        }
    }

    db.save_session_token(&access_token, &github_id, &email, &name, &avatar_url, &installation_id)
        .map_err(|e| e.to_string())?;

    if let Some(central_db) = state.central_db.lock().await.as_ref() {
        let _ = central_db.upsert_user(&github_id, &name, &email, &avatar_url).await;
    }

    Ok(User {
        id: 0,
        github_id,
        email,
        name,
        avatar_url,
    })
}

#[tauri::command]
async fn logout(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().await;
    db.clear_session()
}

#[tauri::command]
async fn save_github_token(token: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let (github_id, email, name, avatar_url) = {
        let auth = state.auth.lock().await;
        auth.get_github_user(&token)
            .await
            .map_err(|e| format!("Invalid token: {}", e))?
    };

    let db = state.db.lock().await;
    db.save_session_token(&token, &github_id, &email, &name, &avatar_url, "")
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn select_folder_dialog(app_handle: tauri::AppHandle) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;
    use std::sync::Arc;
    use tokio::sync::oneshot;

    let (tx, rx) = oneshot::channel();
    let tx = Arc::new(std::sync::Mutex::new(Some(tx)));

    app_handle.dialog().file().pick_folder(move |path| {
        if let Some(tx) = tx.lock().unwrap().take() {
            let _ = tx.send(path);
        }
    });

    match rx.await {
        Ok(Some(path)) => Ok(path.to_string()),
        Ok(None) => Err("No folder selected".to_string()),
        Err(_) => Err("Failed to select folder".to_string()),
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CleanOptionsDto {
    pub mode: String,
    pub include_images: bool,
    pub include_videos: bool,
    pub include_documents: bool,
    pub create_readme: bool,
}

#[tauri::command]
async fn start_cleaning_command(
    source_dir: String,
    output_dir: String,
    state: tauri::State<'_, AppState>,
    options: CleanOptionsDto,
    app_handle: tauri::AppHandle,
) -> Result<cleaner::CleanResult, String> {
    let start_time = std::time::Instant::now();

    let mode = match options.mode.as_str() {
        "flatten" => cleaner::CleanMode::Flatten,
        _ => cleaner::CleanMode::Clean,
    };

    let clean_options = cleaner::CleanOptions {
        mode,
        include_images: options.include_images,
        include_videos: options.include_videos,
        include_documents: options.include_documents,
        create_readme: options.create_readme,
    };

    let user_info = {
        let db = state.db.lock().await;
        db.get_user_info().ok().flatten()
    };

    // Cleaning is CPU/IO-bound; run it on a blocking thread so the UI never freezes.
    let handle = app_handle.clone();
    let result = tokio::task::spawn_blocking(move || {
        cleaner::start_cleaning(&source_dir, &output_dir, clean_options, Some(&handle))
    })
    .await
    .map_err(|e| format!("Cleaning task failed: {}", e))??;

    let execution_time_ms = start_time.elapsed().as_millis() as i32;
    let junk_mb = result.total_size_bytes as f64 / (1024.0 * 1024.0);

    if let Some((github_id, email, name, avatar_url)) = user_info {
        let central_guard = state.central_db.lock().await;
        if let Some(central_db) = central_guard.as_ref() {
            let _ = central_db.upsert_user(&github_id, &name, &email, &avatar_url).await;
            let _ = central_db.log_clean(&github_id, result.total_size_bytes, "clean_operation").await;
            let _ = central_db.update_global_stats(
                0, junk_mb, execution_time_ms, 0.0, 0, 0, true, junk_mb,
            ).await;
        }
    }

    Ok(result)
}

#[tauri::command]
fn scan_project_command(source_dir: String, include_images: bool) -> Result<cleaner::ScanResult, String> {
    let result = cleaner::scan_project(&source_dir, include_images);
    Ok(result)
}

#[tauri::command]
async fn get_user_organizations(state: tauri::State<'_, AppState>) -> Result<Vec<serde_json::Value>, String> {
    let token = {
        let db = state.db.lock().await;
        db.get_session_token()
            .map_err(|e| e.to_string())?
            .ok_or("No GitHub token found. Please login again.")?
    };

    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/user/orgs")
        .bearer_auth(&token)
        .header("User-Agent", "LidBridge")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch organizations: {}", e))?;

    let orgs: Vec<serde_json::Value> = response.json().await.map_err(|e| format!("Failed to parse: {}", e))?;

    Ok(orgs)
}

#[tauri::command]
async fn create_and_push_command(
    state: tauri::State<'_, AppState>,
    path: String,
    config: RepoConfig,
    owner_type: String,
    owner_name: String,
) -> Result<String, String> {
    let start_time = std::time::Instant::now();

    let db = state.db.lock().await;
    let token = db.get_session_token()
        .map_err(|e| e.to_string())?
        .ok_or("No GitHub token found. Please login again.")?;

    let user_info = db.get_user_info()
        .map_err(|e| e.to_string())?
        .ok_or("No user information found. Please login again.")?;

    let (github_id, email, name, avatar_url) = user_info;

    let client = reqwest::Client::new();
    let scope_check = client
        .get("https://api.github.com/user")
        .bearer_auth(&token)
        .header("User-Agent", "LidBridge")
        .send()
        .await
        .map_err(|e| format!("Failed to verify token: {}", e))?;

    if let Some(scopes) = scope_check.headers().get("X-OAuth-Scopes") {
        let scopes_str = scopes.to_str().unwrap_or("");
        log::info!("Current token scopes: {}", scopes_str);

        if !scopes_str.contains("repo") {
            return Err("Token missing 'repo' scope...".to_string());
        }
    }

    let pusher = git::GitPusher::new(&token);
    let result = pusher.create_and_push_with_owner(
        &path,
        config.clone(),
        &owner_type,
        &owner_name,
        &name,
        &email
    ).await?;

    let execution_time_ms = start_time.elapsed().as_millis() as i32;
    log::info!("Push completed in {} ms", execution_time_ms);

    let owner_name_str = if owner_type == "org" { owner_name.clone() } else { github_id.clone() };

    db.save_repo_history(&github_id, &config.name, &result, &owner_type, &owner_name_str)
        .map_err(|e| format!("Failed to save repo history: {}", e))?;

    let central_db_guard = state.central_db.lock().await;
    if let Some(central_db) = central_db_guard.as_ref() {
        match central_db.upsert_user(&github_id, &name, &email, &avatar_url).await {
            Ok(_) => {
                log::info!("User upserted in central db");
                match central_db.log_repo_creation(&github_id, &config.name).await {
                    Ok(_) => log::info!("Repo creation logged to central db"),
                    Err(e) => log::warn!("Failed to log repo creation: {}", e),
                }
                match central_db.update_global_stats(
                    0,
                    0.0,
                    execution_time_ms,
                    0.0,
                    0,
                    0,
                    true,
                    0.0
                ).await {
                    Ok(_) => log::info!("Global stats updated"),
                    Err(e) => log::warn!("Failed to update global stats: {}", e),
                }
            }
            Err(e) => log::warn!("Failed to upsert user: {}", e),
        }
    } else {
        log::warn!("Central database not available, skipping central logging");
    }

    Ok(result)
}

#[tauri::command]
async fn analyze_text_ai(_text: String, _output_path: String) -> Result<(), String> {
    Err("AI analysis feature has been removed".to_string())
}

#[tauri::command]
fn get_global_stats() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({}))
}

// ========== MAIN FUNCTION ==========

pub fn run() {
    dotenv::dotenv().ok();

    let db = db::Database::new().expect("Failed to initialize database");
    let auth = auth::AuthManager::new();

    let central_db = match tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(db::central::CentralDb::new())
    {
        Ok(cdb) => {
            log::info!("Connected to central database");
            Some(cdb)
        }
        Err(e) => {
            log::warn!("Could not connect to central database: {}", e);
            log::warn!("Running WITHOUT central database. Global analytics will be disabled.");
            None
        }
    };

    let app_state = AppState {
        db: Mutex::new(db),
        auth: Mutex::new(auth),
        central_db: Mutex::new(central_db),
        oauth_callback_tx: Mutex::new(None),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_session,
            get_repo_history,
            start_oauth,
            complete_oauth,
            save_github_token,
            logout,
            select_folder_dialog,
            start_cleaning_command,
            scan_project_command,
            create_and_push_command,
            get_user_organizations,
            get_global_stats,
            analyze_text_ai,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

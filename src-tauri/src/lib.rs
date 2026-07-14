#![windows_subsystem = "windows"]

use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use tauri::Emitter;
pub mod auth;
pub mod cleaner;
pub mod git;
pub mod db;
pub mod github_app;
pub mod history_store;
pub mod secret;

pub struct AppState {
    pub auth: Mutex<auth::AuthManager>,
    pub history_store: Mutex<history_store::HistoryStore>,
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
    pub license_template: String,
    pub repo_type: String,
}

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
                            Some(rest.trim_end_matches(" HTTP").to_string())
                        }
                    } else {
                        None
                    };

                    let response = if let Some(code) = code {
                        log::info!("Received OAuth code, exchanging for token...");
                        let _ = app_handle.emit("oauth-code-received", &code);
                        r#"<!DOCTYPE html>
<html>
<head><meta charset="UTF-8"><title>LidBridge</title>
<style>body{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;background:linear-gradient(135deg,#0A0A0F 0%,#1a1a2e 100%);color:white;display:flex;justify-content:center;align-items:center;height:100vh;margin:0}
.success{font-size:48px;margin-bottom:20px}h1{color:#00D4FF;margin-bottom:10px}p{color:#888}</style></head>
<body><div style="text-align:center">
<div style="color:#2ea043;font-size:48px;line-height:1">&#10003;</div>
<h1>Authentication Successful!</h1><p>You can close this window and return to LidBridge.</p>
<script>setTimeout(() => window.close(), 3000)</script>
</div></body></html>"#.to_string()
                    } else {
                        r#"<!DOCTYPE html>
<html>
<head><meta charset="UTF-8"><title>LidBridge</title>
<style>body{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;background:linear-gradient(135deg,#0A0A0F 0%,#1a1a2e 100%);color:white;display:flex;justify-content:center;align-items:center;height:100vh;margin:0}
.error{font-size:48px;margin-bottom:20px}h1{color:#ff4757}</style></head>
<body><div style="text-align:center">
<div style="color:#ff4757;font-size:48px;line-height:1">&#10007;</div>
<h1>Authentication Failed</h1><p>Please try again.</p>
</div></body></html>"#.to_string()
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

#[tauri::command]
async fn get_session(state: tauri::State<'_, AppState>) -> Result<Option<User>, String> {
    let store = state.history_store.lock().await;
    match store.load_session() {
        Ok(Some(session)) => Ok(Some(User {
            id: 0,
            github_id: session.github_id,
            email: session.email,
            name: session.name,
            avatar_url: session.avatar_url,
        })),
        Ok(None) => Ok(None),
        Err(e) => Err(e),
    }
}

#[tauri::command]
async fn get_repo_history(state: tauri::State<'_, AppState>) -> Result<Vec<serde_json::Value>, String> {
    let store = state.history_store.lock().await;
    if store.load_session()?.is_none() {
        return Err("Sign in to GitHub before viewing repository history".to_string());
    }
    let history = store.load_repo_history()?;

    let repos = history.into_iter().map(|entry| {
        serde_json::json!({
            "repo_name": entry.repo_name,
            "repo_url": entry.repo_url,
            "owner_type": entry.owner_type,
            "owner_name": entry.owner_name,
            "created_at": entry.created_at,
        })
    }).collect();

    Ok(repos)
}

#[tauri::command]
async fn start_oauth(_app_handle: tauri::AppHandle) -> Result<String, String> {
    let auth = auth::AuthManager::new();
    let state = uuid::Uuid::new_v4().to_string().replace("-", "");
    let auth_url = auth.build_auth_url(&state);

    let _ = open::that(&auth_url);

    Ok(auth_url)
}

#[tauri::command]
async fn complete_oauth(code: String, state: tauri::State<'_, AppState>) -> Result<User, String> {
    let auth = state.auth.lock().await;

    let access_token = auth.exchange_code_for_token(&code)
        .await
        .map_err(|e| e.to_string())?;

    let (github_id, email, name, avatar_url) = auth.get_github_user(&access_token)
        .await
        .map_err(|e| e.to_string())?;

    let local_avatar = auth.download_avatar(&avatar_url).await.unwrap_or_default();
    let avatar_to_save = if local_avatar.is_empty() { avatar_url.clone() } else { local_avatar };

    drop(auth);

    let history_store = state.history_store.lock().await;
    history_store.save_session(&access_token, &github_id, &email, &name, &avatar_to_save, "")
        .map_err(|e| e.to_string())?;

    Ok(User {
        id: 0,
        github_id,
        email,
        name,
        avatar_url: avatar_to_save,
    })
}

#[tauri::command]
async fn logout(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let history_store = state.history_store.lock().await;
    history_store.clear_session().map_err(|e| e.to_string())?;

    let avatar_dir = dirs::data_local_dir()
        .map(|d| d.join("LidBridge").join("avatars"));
    if let Some(dir) = avatar_dir {
        let _ = std::fs::remove_dir_all(dir);
    }

    Ok(())
}

#[tauri::command]
async fn save_github_token(token: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let (github_id, email, name, avatar_url) = {
        let auth = state.auth.lock().await;
        auth.get_github_user(&token)
            .await
            .map_err(|e| format!("Invalid token: {}", e))?
    };

    let local_avatar = {
        let auth = state.auth.lock().await;
        auth.download_avatar(&avatar_url).await.unwrap_or_default()
    };
    let avatar_to_save = if local_avatar.is_empty() { avatar_url } else { local_avatar };

    let history_store = state.history_store.lock().await;
    history_store.save_session(&token, &github_id, &email, &name, &avatar_to_save, "")
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
    pub secret_replacements: Vec<cleaner::SecretReplacement>,
}

#[tauri::command]
async fn start_cleaning_command(
    source_dir: String,
    output_dir: String,
    _state: tauri::State<'_, AppState>,
    options: CleanOptionsDto,
    app_handle: tauri::AppHandle,
) -> Result<cleaner::CleanResult, String> {
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
        secret_replacements: options.secret_replacements,
    };

    let handle = app_handle.clone();
    let result = tokio::task::spawn_blocking(move || {
        cleaner::start_cleaning(&source_dir, &output_dir, clean_options, Some(&handle))
    })
    .await
    .map_err(|e| format!("Cleaning task failed: {}", e))??;

    Ok(result)
}

#[tauri::command]
async fn scan_project_command(source_dir: String, include_images: bool, app_handle: tauri::AppHandle) -> Result<cleaner::ScanResult, String> {
    tokio::task::spawn_blocking(move || {
        cleaner::scan_project_with_progress(&source_dir, include_images, Some(&app_handle))
    })
    .await
    .map_err(|e| format!("Scanning task failed: {}", e))
}

#[tauri::command]
async fn get_user_organizations(state: tauri::State<'_, AppState>) -> Result<Vec<serde_json::Value>, String> {
    let token = {
        let store = state.history_store.lock().await;
        store.load_session()
            .map_err(|e| e.to_string())?
            .ok_or("No GitHub token found. Please login again.")?
            .access_token
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
    let session = {
        let store = state.history_store.lock().await;
        store.load_session()
            .map_err(|e| e.to_string())?
            .ok_or("No GitHub session found. Please login again.")?
    };

    let token = session.access_token;
    let github_id = session.github_id.clone();
    let email = session.email.clone();
    let name = session.name.clone();

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

        let allowed_scopes = ["repo", "repo:status", "repo_deployment", "public_repo", "repo:invite", "security_events"];
        let has_valid_scope = scopes_str.split(',')
            .map(|s| s.trim())
            .any(|s| allowed_scopes.contains(&s));
        if !has_valid_scope {
            return Err(format!(
                "Token lacks push permissions.\n\n\
                 Current scopes: {}\n\n\
                 Solution: Click 'Use Personal Token' on the login screen and enter a classic token with 'repo' scope.\n\
                 Create one at: https://github.com/settings/tokens/new", scopes_str
            ));
        }
    } else {
        log::warn!("No X-OAuth-Scopes header — proceeding but token may lack permissions");
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

    let owner_name_str = if owner_type == "org" { owner_name.clone() } else { github_id.clone() };

    let store = state.history_store.lock().await;
    store.save_repo_history(&config.name, &result, &owner_type, &owner_name_str)
        .map_err(|e| format!("Failed to save repo history: {}", e))?;

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

pub fn run() {
    let _ = dotenv::dotenv();

    let auth = auth::AuthManager::new();
    let history_store = history_store::HistoryStore::new().expect("Failed to initialize encrypted history store");

    let app_state = AppState {
        auth: Mutex::new(auth),
        history_store: Mutex::new(history_store),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .manage(app_state)
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let _ = start_oauth_server(handle, 2026).await;
            });
            Ok(())
        })
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

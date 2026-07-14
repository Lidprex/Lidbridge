use std::collections::HashSet;
use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use tauri::{AppHandle, Emitter};
use serde::{Deserialize, Serialize};
use regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretReplacement {
    pub name: String,
    pub replacement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanProgress {
    pub phase: String,
    pub current: usize,
    pub total: usize,
    pub current_file: String,
    pub percentage: u8,
    pub bytes_copied: u64,
    pub deleted_count: usize,
}

const SKIP_DIRS: &[&str] = &[
    ".git", ".svn", ".hg", ".bzr",
    ".idea", ".vscode", ".vs", ".sublime-project", ".sublime-workspace",
    ".eclipse", ".settings", ".project", ".classpath",
    "__pycache__", ".pytest_cache", ".mypy_cache", ".coverage",
    ".tox", ".hypothesis", ".eggs", ".egg-info", ".venv", "venv", ".env", "ENV", "virtualenv",
    "node_modules", ".npm", ".yarn", ".pnp", ".pnpm-store", "bower_components",
    "dist", "build", "target", "out", "bin", "obj", "release", "debug",
    ".next", ".nuxt", ".gatsby", "next", "nuxt", ".angular", ".cache",
    ".gradle", ".m2", "gradle", "buildSrc",
    "Pods", ".xcworkspace", ".xcodeproj", "DerivedData", "AndroidStudioProjects",
    "site-packages", "dist-packages", ".nyc_output", "__MACOSX",
    "temp", "tmp", "logs", ".logs", "backup",
];

const SKIP_EXTENSIONS: &[&str] = &[
    ".log", ".tmp", ".temp", ".bak", ".swp", ".swo", ".cache",
    ".pyc", ".pyo", ".pyd",
    ".so", ".dll", ".dylib", ".class", ".jar", ".war", ".ear",
    ".o", ".a", ".lib", ".obj", ".exe", ".msi",
    ".zip", ".tar", ".gz", ".rar", ".7z", ".bz2", ".xz", ".iso",
    ".mp4", ".mp3", ".avi", ".mov", ".mkv", ".flv", ".wmv", ".m4a", ".flac", ".wav",
    ".ttf", ".woff", ".woff2", ".eot", ".otf", ".pdf", ".doc", ".docx",
    ".lock", ".pid",
];

const SKIP_FILES: &[&str] = &[
    ".DS_Store", "Thumbs.db", "desktop.ini", ".Spotlight-V100",
    ".Trashes", "ehthumbs.db", "Folder.jpg",
    "package-lock.json", "yarn.lock", "pnpm-lock.yaml", "composer.lock", "Gemfile.lock",
    "Cargo.lock", "poetry.lock",
    ".env", ".env.local", ".env.production", ".env.development", ".env.test",
    ".gitkeep", ".keep", ".empty",
];

const IMAGE_EXTENSIONS: &[&str] = &[
    ".png", ".jpg", ".jpeg", ".gif", ".ico", ".svg", ".webp", ".bmp",
    ".tiff", ".tif", ".raw", ".cr2", ".nef", ".heic", ".avif",
];

const VIDEO_EXTENSIONS: &[&str] = &[
    ".mp4", ".mkv", ".avi", ".mov", ".wmv", ".flv", ".webm", ".m4v", ".mpg", ".mpeg",
];

const DOCUMENT_EXTENSIONS: &[&str] = &[
    ".pdf", ".doc", ".docx", ".xls", ".xlsx", ".ppt", ".pptx", ".odt", ".ods", ".odp",
];

const SECRET_PATTERNS: &[(&str, &str)] = &[
    (r#"ghp_[a-zA-Z0-9]{36}"#, "GitHub Personal Access Token"),
    (r#"gho_[a-zA-Z0-9]{36}"#, "GitHub OAuth Token"),
    (r#"ghu_[a-zA-Z0-9]{36}"#, "GitHub User Token"),
    (r#"ghs_[a-zA-Z0-9]{36}"#, "GitHub Server Token"),
    (r#"sk-[a-zA-Z0-9]{48}"#, "OpenAI API Key"),
    (r#"sk-proj-[a-zA-Z0-9]{48}"#, "OpenAI Project Key"),
    (r#"AKIA[0-9A-Z]{16}"#, "AWS Access Key"),
    (r#"AIza[0-9A-Za-z\\-_]{35}"#, "Google API Key"),
    (r#"sk_live_[a-zA-Z0-9]{24}"#, "Stripe Secret Key"),
    (r#"rk_live_[a-zA-Z0-9]{24}"#, "Stripe Restricted Key"),
    (r#"MTA[0-9]{17}.[0-9A-Za-z_-]{6}"#, "Discord Bot Token"),
    (r#"(?i)api[_-]?key[_-]?=[a-zA-Z0-9]{16,}"#, "Generic API Key"),
    (r#"(?i)secret[_-]?key[_-]?=[a-zA-Z0-9]{16,}"#, "Generic Secret Key"),
    (r#"(?i)password[_-]?=[a-zA-Z0-9]{8,}"#, "Plaintext Password"),
    (r#"(?i)token[_-]?=[a-zA-Z0-9]{16,}"#, "Access Token"),
    (r#"xox[bpsa]-[0-9a-zA-Z-]{10,}"#, "Slack Token"),
    (r#"(?i)client_secret[_-]?[=:]\s*['\"]?[a-zA-Z0-9_\-]{16,}['\"]?"#, "OAuth Client Secret"),
    (r#"(?i)private[_-]?key[_-]?[=:]"#, "Private Key Reference"),
    (r#"-----BEGIN (RSA |EC |DSA )?PRIVATE KEY-----"#, "Private Key File"),
    (r#"eyJ[a-zA-Z0-9_-]{10,}\.[a-zA-Z0-9_-]{10,}\.[a-zA-Z0-9_-]{10,}"#, "JWT Token"),
    (r#"(?i)(aws_secret_access_key|AWS_SECRET_ACCESS_KEY)[=:]\s*['\"]?[a-zA-Z0-9/+=]{40}['\"]?"#, "AWS Secret Access Key"),
    (r#"(?i)DATABASE_URL[_=]"#, "Database Connection String"),
    (r#"(?i)(ftp|http|https)://[^:]+:[^@]+@"#, "Embedded Credentials in URL"),
];

const SECRET_FILE_NAMES: &[&str] = &[
    ".env",
    ".env.local",
    ".env.production",
    ".env.development",
    ".env.test",
    ".env.staging",
    ".env.backup",
    ".env.old",
    "credentials.json",
    "credentials.yaml",
    "credentials.yml",
    "credentials.xml",
    "secrets.json",
    "secrets.yaml",
    "secrets.yml",
    "secret.txt",
    "secret.key",
    "service-account.json",
    "service-account.yaml",
    "firebase-service-account.json",
    "gcloud-service-key.json",
    ".htpasswd",
    ".htaccess",
    "id_rsa",
    "id_dsa",
    "id_ecdsa",
    "id_ed25519",
    "id_rsa.pub",
    "id_ed25519.pub",
    "deploy_key",
    "deploy_key.pub",
];

const SECRET_FILE_EXTENSIONS: &[&str] = &[
    ".pem",
    ".p12",
    ".pfx",
    ".jks",
    ".keystore",
    ".key",
    ".crt",
    ".cert",
    ".p7b",
    ".p7c",
    ".keystore",
];

pub fn should_skip_dir(name: &str) -> bool {
    let name_lower = name.to_lowercase();
    SKIP_DIRS.iter().any(|&dir| {
        dir == name || dir.to_lowercase() == name_lower
    })
}

pub fn detect_secrets(content: &str) -> Vec<String> {
    let mut found = Vec::new();

    for (pattern, name) in SECRET_PATTERNS {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(content) {
                found.push(name.to_string());
            }
        }
    }

    found
}

pub fn detect_secrets_in_filename(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_str()?;

    for &secret_name in SECRET_FILE_NAMES {
        if name.eq_ignore_ascii_case(secret_name) {
            return Some(format!("Secret file detected: {}", name));
        }
    }

    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let ext_lower = format!(".{}", ext.to_lowercase());
        for &secret_ext in SECRET_FILE_EXTENSIONS {
            if ext_lower == secret_ext {
                return Some(format!("Potentially sensitive file ({}): {}", ext_lower, name));
            }
        }
    }

    None
}

pub fn should_skip_file(path: &Path, include_images: bool, include_videos: bool, include_documents: bool) -> bool {
    let name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    if SKIP_FILES.contains(&name) {
        return true;
    }

    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let ext_lower = ext.to_lowercase();
        let ext_with_dot = format!(".{}", ext_lower);

        if SKIP_EXTENSIONS.contains(&ext_with_dot.as_str()) {
            return true;
        }

        if !include_images && IMAGE_EXTENSIONS.contains(&ext_with_dot.as_str()) {
            return true;
        }

        if !include_videos && VIDEO_EXTENSIONS.contains(&ext_with_dot.as_str()) {
            return true;
        }

        if !include_documents && DOCUMENT_EXTENSIONS.contains(&ext_with_dot.as_str()) {
            return true;
        }
    }

    false
}

fn first_skip_dir<'a>(parts: &'a [&'a str]) -> Option<&'a str> {
    for p in parts.iter().take(parts.len().saturating_sub(1)) {
        if should_skip_dir(p) {
            return Some(p);
        }
    }
    None
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScanResult {
    pub total_files: usize,
    pub clean_files: usize,
    pub skipped_dirs: usize,
    pub total_lines: usize,
    pub skipped_files: usize,
    pub total_size: u64,
    pub clean_size: u64,
    pub project_type: String,
    pub skippable: std::collections::HashMap<String, u64>,
    pub secrets_count: usize,
    pub secret_matches: Vec<String>,
    pub secret_suggestions: Vec<String>,
}

pub fn detect_project_type(path: &Path) -> String {
    let checks = [
        (vec!["package.json"], "Node.js / JavaScript"),
        (vec!["requirements.txt", "setup.py", "pyproject.toml", "Pipfile"], "Python"),
        (vec!["pom.xml"], "Java (Maven)"),
        (vec!["build.gradle", "build.gradle.kts"], "Java (Gradle)"),
        (vec!["pubspec.yaml"], "Flutter / Dart"),
        (vec!["composer.json"], "PHP"),
        (vec!["Gemfile"], "Ruby"),
        (vec!["go.mod"], "Go"),
        (vec!["Cargo.toml"], "Rust"),
        (vec!["CMakeLists.txt"], "C / C++ (CMake)"),
        (vec!["Makefile", "makefile"], "C / C++ (Make)"),
        (vec!["mix.exs"], "Elixir"),
        (vec!["project.clj"], "Clojure"),
        (vec!["build.sbt"], "Scala"),
        (vec!["*.csproj"], ".NET / C#"),
        (vec!["Podfile"], "iOS (CocoaPods)"),
    ];

    for (files, ptype) in checks {
        if files.iter().any(|f| path.join(f).exists()) {
            return ptype.to_string();
        }
    }

    if path.join("src").exists() && path.join("lib").exists() {
        return "Generic (src/lib structure)".to_string();
    }

    "Generic Project".to_string()
}

fn secret_suggestions_from_matches(matches: &[String]) -> Vec<String> {
    matches.iter().map(|match_name| derive_placeholder_for_secret(match_name)).collect()
}

pub fn derive_placeholder_for_secret(match_name: &str) -> String {
    let normalized = match_name.to_lowercase();
    if normalized.contains("github") && normalized.contains("client") && normalized.contains("id") {
        return "your_github_client_id".to_string();
    }
    if normalized.contains("github") && normalized.contains("client") && normalized.contains("secret") {
        return "your_github_client_secret".to_string();
    }
    if normalized.contains("github") && normalized.contains("token") {
        return "your_github_token".to_string();
    }
    if normalized.contains("openai") || normalized.contains("api key") {
        return "your_api_key".to_string();
    }
    if normalized.contains("aws") {
        return "your_aws_access_key".to_string();
    }
    if normalized.contains("password") {
        return "your_password".to_string();
    }
    if normalized.contains("secret") {
        return "your_secret".to_string();
    }
    "your_placeholder".to_string()
}

fn replace_secret_content(content: &str, replacements: &[SecretReplacement]) -> String {
    let mut updated = content.to_string();
    for replacement in replacements {
        if replacement.replacement.trim().is_empty() {
            continue;
        }
        let pattern = match replacement.name.to_lowercase().as_str() {
            name if name.contains("github") && name.contains("token") => r#"gh[poush]_[A-Za-z0-9]{36}"#,
            name if name.contains("openai") || name.contains("api key") => r#"sk-[A-Za-z0-9]{48}|sk-proj-[A-Za-z0-9]{48}"#,
            name if name.contains("aws") => r#"AKIA[0-9A-Z]{16}"#,
            name if name.contains("google") => r#"AIza[0-9A-Za-z\-_]{35}"#,
            name if name.contains("password") => r#"(?i)(password|passwd|pwd)[^\n=]*=[^\n]+"#,
            name if name.contains("secret") => r#"(?i)(secret|token)[^\n=]*=[^\n]+"#,
            _ => continue,
        };
        if let Ok(re) = Regex::new(pattern) {
            updated = re.replace_all(&updated, replacement.replacement.as_str()).into_owned();
        }
    }
    updated
}

pub fn scan_project(source_dir: &str, include_images: bool) -> ScanResult {
    scan_project_with_progress(source_dir, include_images, None)
}

pub fn scan_project_with_progress(source_dir: &str, include_images: bool, app_handle: Option<&AppHandle>) -> ScanResult {
    let path = Path::new(source_dir);
    let mut result = ScanResult::default();

    let mut total_lines = 0;
    let mut secrets_count = 0;

    result.project_type = detect_project_type(path);

    let mut seen_dirs: HashSet<String> = HashSet::new();
    let mut walker = WalkDir::new(path).follow_links(false).into_iter();
    let mut scanned_files = 0usize;

    while let Some(entry) = walker.next() {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let item_path = entry.path();

        if item_path.is_dir() {
            let name = item_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            if item_path != path && should_skip_dir(name) {
                walker.skip_current_dir();
                if seen_dirs.contains(name) {
                    continue;
                }
                seen_dirs.insert(name.to_string());
                result.skipped_dirs += 1;

                let dir_size: u64 = WalkDir::new(item_path)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().is_file())
                    .filter_map(|e| e.metadata().ok())
                    .map(|m| m.len())
                    .sum();
                result.skippable.insert(name.to_string(), dir_size);
                result.total_size += dir_size;

                let dir_file_count: usize = WalkDir::new(item_path)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().is_file())
                    .count();
                result.skipped_files += dir_file_count;
            }
            continue;
        }

        if !item_path.is_file() {
            continue;
        }

        scanned_files += 1;
        let size = match item_path.metadata() {
            Ok(m) => m.len(),
            Err(_) => 0,
        };

        if let Ok(content) = std::fs::read_to_string(item_path) {
            total_lines += content.lines().count();

            let secrets = detect_secrets(&content);
            secrets_count += secrets.len();
            if !secrets.is_empty() {
                result.secret_matches.extend(secrets);
            }
        }

        if let Some(secret_desc) = detect_secrets_in_filename(item_path) {
            if !result.secret_matches.contains(&secret_desc) {
                result.secret_matches.push(secret_desc);
                secrets_count += 1;
            }
        }

        result.total_files += 1;
        result.total_size += size;

        let rel = match item_path.strip_prefix(path) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let parts: Vec<&str> = rel.iter()
            .filter_map(|p| p.to_str())
            .collect();

        let skip_dir = first_skip_dir(&parts);
        let skip = skip_dir.is_some()
            || should_skip_file(item_path, include_images, false, false);

        if skip {
            result.skipped_files += 1;
            if let Some(dir) = skip_dir {
                let current = result.skippable.get(dir).copied().unwrap_or(0);
                result.skippable.insert(dir.to_string(), current + size);
            }
        } else {
            result.clean_files += 1;
            result.clean_size += size;
        }

        if let Some(handle) = app_handle {
            let current_file = item_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            if scanned_files % 25 == 0 {
                send_progress(Some(handle), "scanning", scanned_files, 0, &current_file, 0, 0, 0);
            }
        }
    }

    if let Some(handle) = app_handle {
        send_progress(Some(handle), "scanning", scanned_files, scanned_files, "", 100, 0, 0);
    }

    result.secret_matches.sort();
    result.secret_matches.dedup();
    result.secret_suggestions = secret_suggestions_from_matches(&result.secret_matches);
    result.total_lines = total_lines;
    result.secrets_count = secrets_count;

    result
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CleanMode {
    Flatten,
    Clean,
    Scan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanOptions {
    pub mode: CleanMode,
    pub include_images: bool,
    pub include_videos: bool,
    pub include_documents: bool,
    pub create_readme: bool,
    pub secret_replacements: Vec<SecretReplacement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanResult {
    pub success: bool,
    pub cleaned_path: String,
    pub copied_files: usize,
    pub skipped_files: usize,
    pub deleted_items: Vec<String>,
    pub warnings: Vec<String>,
    pub total_size_bytes: u64,
    pub scan_result: ScanResult,
}

fn send_progress(
    app_handle: Option<&AppHandle>,
    phase: &str,
    current: usize,
    total: usize,
    current_file: &str,
    percentage: u8,
    bytes_copied: u64,
    deleted_count: usize,
) {
    if let Some(handle) = app_handle {
        let progress = CleanProgress {
            phase: phase.to_string(),
            current,
            total,
            current_file: current_file.to_string(),
            percentage,
            bytes_copied,
            deleted_count,
        };

        let _ = handle.emit("cleaning-progress", progress);
    }
}

pub fn start_cleaning(
    source_dir: &str,
    output_dir: &str,
    options: CleanOptions,
    app_handle: Option<&AppHandle>,
) -> Result<CleanResult, String> {
    let source = Path::new(source_dir);
    let target = Path::new(output_dir);

    if !source.exists() {
        return Err("Source directory does not exist".to_string());
    }

    if let Err(e) = fs::create_dir_all(target) {
        return Err(format!("Failed to create output directory: {}", e));
    }

    send_progress(app_handle, "scanning", 0, 0, "", 0, 0, 0);

    let scan_result = scan_project_with_progress(source_dir, options.include_images, app_handle);
    let total_files = scan_result.clean_files;

    log::info!("Scanning complete: {} clean files out of {} total",
        total_files, scan_result.total_files);

    send_progress(app_handle, "copying", 0, total_files, "", 0, 0, 0);

    let mut copied = 0;
    let mut skipped = 0;
    let mut bytes_copied = 0u64;
    let mut deleted_items = Vec::new();
    let mut warnings = Vec::new();
    let mut name_counter = std::collections::HashMap::<String, usize>::new();
    let mut last_emit = std::time::Instant::now();

    let mut walker = WalkDir::new(source).follow_links(false).into_iter();
    while let Some(entry) = walker.next() {
        let entry = match entry { Ok(entry) => entry, Err(err) => { warnings.push(err.to_string()); continue; } };
        let item = entry.path();
        if entry.file_type().is_dir() {
            if item != source && should_skip_dir(item.file_name().and_then(|n| n.to_str()).unwrap_or("")) {
                walker.skip_current_dir();
            }
            continue;
        }
        if !entry.file_type().is_file() { continue; }

        let rel = match item.strip_prefix(source) { Ok(rel) => rel, Err(_) => { skipped += 1; continue; } };
        let parts: Vec<&str> = rel.iter().filter_map(|p| p.to_str()).collect();
        if first_skip_dir(&parts).is_some() || should_skip_file(item, options.include_images, options.include_videos, options.include_documents) {
            skipped += 1;
            continue;
        }

        let dest = match options.mode {
            CleanMode::Flatten => {
                let name = item.file_name().and_then(|n| n.to_str()).unwrap_or("file");
                let count = name_counter.entry(name.to_string()).or_insert(0);
                *count += 1;
                if *count == 1 { target.join(name) } else {
                    let stem = Path::new(name).file_stem().and_then(|s| s.to_str()).unwrap_or("file");
                    let ext = Path::new(name).extension().and_then(|e| e.to_str()).unwrap_or("");
                    target.join(if ext.is_empty() { format!("{}__{}", stem, *count - 1) } else { format!("{}__{}.{}", stem, *count - 1, ext) })
                }
            }
            CleanMode::Clean | CleanMode::Scan => target.join(rel),
        };

        let copy_result = dest.parent().ok_or_else(|| "Destination has no parent directory".to_string())
            .and_then(|parent| fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e)))
            .and_then(|_| {
                let mut bytes = 0u64;
                if item.is_file() {
                    let original = fs::read(item).map_err(|e| format!("Failed to read source file: {}", e))?;
                    let content_str = String::from_utf8_lossy(&original).to_string();
                    let updated_content = replace_secret_content(&content_str, &options.secret_replacements);
                    if updated_content != content_str {
                        fs::write(&dest, updated_content).map_err(|e| format!("Failed to write cleaned file: {}", e))?;
                    } else {
                        fs::copy(item, &dest).map_err(|e| format!("Failed to copy: {}", e))?;
                    }
                    bytes = original.len() as u64;
                }
                Ok(bytes)
            });
        match copy_result {
            Ok(size) => { copied += 1; bytes_copied += size; }
            Err(err) => { skipped += 1; warnings.push(format!("Skipped {}: {}", item.display(), err)); continue; }
        }
        if last_emit.elapsed().as_millis() > 50 || copied == total_files {
            last_emit = std::time::Instant::now();
            let percentage = if total_files > 0 { ((copied as f64 / total_files as f64) * 100.0) as u8 } else { 100 };
            send_progress(app_handle, "copying", copied, total_files, item.file_name().and_then(|n| n.to_str()).unwrap_or(""), percentage, bytes_copied, 0);
        }
    }

    if options.mode == CleanMode::Clean {
        send_progress(app_handle, "cleaning", 0, 0, "", 0, bytes_copied, 0);

        for entry in WalkDir::new(target)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            if path.is_dir() {
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                if should_skip_dir(name) {
                    if let Err(e) = fs::remove_dir_all(path) {
                        log::warn!("Failed to remove directory {:?}: {}", path, e);
                        warnings.push(format!("Could not remove {}: {}", name, e));
                    } else {
                        deleted_items.push(format!("{}/", name));
                    }
                }
            } else if path.is_file() && should_skip_file(path, options.include_images, options.include_videos, options.include_documents) {
                if let Err(e) = fs::remove_file(path) {
                    log::warn!("Failed to remove file {:?}: {}", path, e);
                } else {
                    let name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("file");
                    deleted_items.push(name.to_string());
                }
            }
        }
    }

    if target.join(".env").exists() {
        warnings.push("Warning: .env file detected! Exclude sensitive data before pushing.".to_string());
    }

    if target.join(".git/config").exists() {
        warnings.push("Warning: Git configuration detected. Consider removing .git folder if not needed.".to_string());
    }

    if options.create_readme {
        let readme_path = target.join("README.md");
        if !readme_path.exists() {
            let project_name = target.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Project");
            let clean_name = project_name.trim_end_matches("_LidBridge");
            let readme_content = format!(
                "# {}\n\nThis project was cleaned and prepared for publishing using [LidBridge](https://github.com/lidprex/LidBridge).\n",
                clean_name
            );
            if let Err(e) = fs::write(&readme_path, readme_content) {
                warnings.push(format!("Failed to create README.md: {}", e));
            } else {
                copied += 1;
            }
        }
    }

    send_progress(
        app_handle,
        "complete",
        copied,
        total_files,
        "",
        100,
        bytes_copied,
        deleted_items.len(),
    );

    log::info!("Cleaning complete: {} copied, {} skipped, {} deleted",
        copied, skipped, deleted_items.len());

    Ok(CleanResult {
        success: true,
        cleaned_path: output_dir.to_string(),
        copied_files: copied,
        skipped_files: skipped,
        deleted_items,
        warnings,
        total_size_bytes: bytes_copied,
        scan_result,
    })
}

pub async fn analyze_text_with_ai(_text: &str, _output_path: &Path) -> Result<(), String> {
    Err("AI analysis feature is not yet available".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skipped_directories_contribute_to_scan_size() {
        let base_dir = std::env::temp_dir().join(format!("lidbridge-scan-skip-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base_dir);
        fs::create_dir_all(base_dir.join("node_modules")).unwrap();
        fs::create_dir_all(base_dir.join("src")).unwrap();
        fs::write(base_dir.join("node_modules").join("package.js"), "console.log('hello')").unwrap();
        fs::write(base_dir.join("src").join("app.js"), "console.log('ok')").unwrap();

        let result = scan_project(base_dir.to_str().unwrap(), false);

        assert!(result.skipped_dirs > 0);
        assert!(result.skippable.get("node_modules").copied().unwrap_or(0) > 0);

        let _ = fs::remove_dir_all(base_dir);
    }

    #[test]
    fn test_should_skip_dir() {
        assert!(should_skip_dir("node_modules"));
        assert!(should_skip_dir(".git"));
        assert!(should_skip_dir("__pycache__"));
        assert!(!should_skip_dir("src"));
    }

    #[test]
    fn test_should_skip_file() {
        let path = Path::new("test.pyc");
        assert!(should_skip_file(path, false, false, false));

        let path = Path::new("test.png");
        assert!(should_skip_file(path, false, false, false));
        assert!(!should_skip_file(path, true, false, false));
    }

    #[test]
    fn test_placeholder_suggestions_are_generated() {
        assert_eq!(derive_placeholder_for_secret("GitHub Personal Access Token"), "your_github_token");
        assert_eq!(derive_placeholder_for_secret("OpenAI API Key"), "your_api_key");
        assert_eq!(derive_placeholder_for_secret("AWS Access Key"), "your_aws_access_key");
    }
}

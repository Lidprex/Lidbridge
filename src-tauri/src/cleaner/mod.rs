// LidBridge — Open-Source Desktop Tool for Cleaning and Publishing Projects to GitHub
// Copyright (C) 2026 Lidprex Labs <https://lidprex.onrender.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::fs;
use walkdir::WalkDir;
use rayon::prelude::*;
use tauri::{AppHandle, Emitter};
use serde::{Deserialize, Serialize};
use regex::Regex;

/// Progress event sent to the frontend during cleaning operations.
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

/// Directories that should be excluded from clean output.
const SKIP_DIRS: &[&str] = &[
    // Version Control
    ".git", ".svn", ".hg", ".bzr",
    // IDEs & Editors
    ".idea", ".vscode", ".vs", ".sublime-project", ".sublime-workspace",
    ".eclipse", ".settings", ".project", ".classpath",
    // Python
    "__pycache__", ".pytest_cache", ".mypy_cache", ".coverage",
    ".tox", ".hypothesis", ".eggs", ".egg-info", ".venv", "venv", ".env", "ENV", "virtualenv",
    // Node.js / JavaScript
    "node_modules", ".npm", ".yarn", ".pnp", ".pnpm-store", "bower_components",
    // Build outputs
    "dist", "build", "target", "out", "bin", "obj", "release", "debug",
    // Frameworks
    ".next", ".nuxt", ".gatsby", "next", "nuxt", ".angular", ".cache",
    // Java / Kotlin
    ".gradle", ".m2", "gradle", "buildSrc",
    // Mobile
    "Pods", ".xcworkspace", ".xcodeproj", "DerivedData", "AndroidStudioProjects",
    // Misc
    "site-packages", "dist-packages", ".nyc_output", "__MACOSX",
    "temp", "tmp", "logs", ".logs", "backup",
];

/// File extensions that are excluded from clean output.
const SKIP_EXTENSIONS: &[&str] = &[
    // Logs & Temp
    ".log", ".tmp", ".temp", ".bak", ".swp", ".swo", ".cache",
    // Python
    ".pyc", ".pyo", ".pyd",
    // Compiled
    ".so", ".dll", ".dylib", ".class", ".jar", ".war", ".ear",
    ".o", ".a", ".lib", ".obj", ".exe", ".msi",
    // Archives
    ".zip", ".tar", ".gz", ".rar", ".7z", ".bz2", ".xz", ".iso",
    // Media
    ".mp4", ".mp3", ".avi", ".mov", ".mkv", ".flv", ".wmv", ".m4a", ".flac", ".wav",
    // Fonts & Documents
    ".ttf", ".woff", ".woff2", ".eot", ".otf", ".pdf", ".doc", ".docx",
    // Lock files
    ".lock", ".pid",
];

/// Specific filenames that are excluded from clean output.
const SKIP_FILES: &[&str] = &[
    // OS artifacts
    ".DS_Store", "Thumbs.db", "desktop.ini", ".Spotlight-V100",
    ".Trashes", "ehthumbs.db", "Folder.jpg",
    // Lock files
    "package-lock.json", "yarn.lock", "pnpm-lock.yaml", "composer.lock", "Gemfile.lock",
    "Cargo.lock", "poetry.lock",
    // Environment files
    ".env", ".env.local", ".env.production", ".env.development", ".env.test",
    // Placeholders
    ".gitkeep", ".keep", ".empty",
];

/// Image file extensions (included or excluded based on user option).
const IMAGE_EXTENSIONS: &[&str] = &[
    ".png", ".jpg", ".jpeg", ".gif", ".ico", ".svg", ".webp", ".bmp",
    ".tiff", ".tif", ".raw", ".cr2", ".nef", ".heic", ".avif",
];

/// Video file extensions (included or excluded based on user option).
const VIDEO_EXTENSIONS: &[&str] = &[
    ".mp4", ".mkv", ".avi", ".mov", ".wmv", ".flv", ".webm", ".m4v", ".mpg", ".mpeg",
];

/// Document file extensions (included or excluded based on user option).
const DOCUMENT_EXTENSIONS: &[&str] = &[
    ".pdf", ".doc", ".docx", ".xls", ".xlsx", ".ppt", ".pptx", ".odt", ".ods", ".odp",
];

/// Regex patterns used to detect exposed secrets in source files.
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
];

/// Returns true if the given directory name should be skipped.
pub fn should_skip_dir(name: &str) -> bool {
    let name_lower = name.to_lowercase();
    SKIP_DIRS.iter().any(|&dir| {
        dir == name || dir.to_lowercase() == name_lower
    })
}

/// Scans file content for known secret patterns and returns matches.
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

/// Returns true if the given file should be excluded from clean output.
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

/// Summary statistics returned after scanning a project directory.
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
}

/// Detects the project type based on common manifest files.
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

/// Scans a project directory and returns statistics without modifying any files.
pub fn scan_project(source_dir: &str, include_images: bool) -> ScanResult {
    let path = Path::new(source_dir);
    let mut result = ScanResult::default();

    let mut total_lines = 0;
    let mut secrets_count = 0;

    result.project_type = detect_project_type(path);

    let _skip_ext: HashSet<&str> = SKIP_EXTENSIONS.iter().copied().collect();
    let image_ext: HashSet<&str> = IMAGE_EXTENSIONS.iter().copied().collect();

    let _skip_ext_strings: HashSet<String> = if include_images {
        _skip_ext.iter().map(|s| s.to_string()).collect()
    } else {
        _skip_ext.union(&image_ext).map(|s| s.to_string()).collect()
    };

    let mut seen_dirs: HashSet<String> = HashSet::new();

    for entry in WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let item_path = entry.path();

        if item_path.is_dir() {
            let name = item_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            if should_skip_dir(name) && !seen_dirs.contains(name) {
                seen_dirs.insert(name.to_string());
                result.skipped_dirs += 1;

                let size: u64 = WalkDir::new(item_path)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_file())
                    .filter_map(|e| e.metadata().ok())
                    .map(|m| m.len())
                    .sum();

                result.skippable.insert(name.to_string(), size);
            }
            continue;
        }

        if !item_path.is_file() {
            continue;
        }

        if let Ok(content) = std::fs::read_to_string(item_path) {
            total_lines += content.lines().count();

            let secrets = detect_secrets(&content);
            secrets_count += secrets.len();
        }

        let size = match item_path.metadata() {
            Ok(m) => m.len(),
            Err(_) => 0,
        };

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
        } else {
            result.clean_files += 1;
            result.clean_size += size;
        }
    }

    result.total_lines = total_lines;
    result.secrets_count = secrets_count;

    result
}

/// Cleaning mode selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CleanMode {
    Flatten,
    Clean,
    Scan,
}

/// Options passed to the cleaning operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanOptions {
    pub mode: CleanMode,
    pub include_images: bool,
    pub include_videos: bool,
    pub include_documents: bool,
    pub create_readme: bool,
}

/// Result returned after a cleaning operation completes.
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

/// Copies clean files from `source_dir` to `output_dir`, skipping junk directories and files.
///
/// This is CPU/IO-bound (walkdir + rayon + fs) and contains no async work, so it is a
/// synchronous function. Callers on an async runtime should run it via
/// `tokio::task::spawn_blocking` to avoid blocking the runtime / UI thread.
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

    let scan_result = scan_project(source_dir, options.include_images);
    let total_files = scan_result.clean_files;

    log::info!("Scanning complete: {} clean files out of {} total",
        total_files, scan_result.total_files);

    send_progress(app_handle, "copying", 0, total_files, "", 0, 0, 0);

    let mut copied = 0;
    let mut skipped = 0;
    let mut bytes_copied = 0u64;
    let mut deleted_items = Vec::new();
    let mut warnings = Vec::new();
    let name_counter: Arc<Mutex<std::collections::HashMap<String, usize>>> = Arc::new(Mutex::new(std::collections::HashMap::new()));

    let files: Vec<PathBuf> = WalkDir::new(source)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();

    let results: Vec<(PathBuf, Result<u64, String>)> = files
        .par_iter()
        .filter_map(|item| {
            let rel = item.strip_prefix(source).ok()?;
            let parts: Vec<&str> = rel.iter()
                .filter_map(|p| p.to_str())
                .collect();

            if first_skip_dir(&parts).is_some() {
                return None;
            }

            if should_skip_file(item, options.include_images, options.include_videos, options.include_documents) {
                return None;
            }

            Some((item.clone(), rel))
        })
        .map(|(item, rel)| {
            let dest = match options.mode {
                CleanMode::Flatten => {
                    let name = item.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("file");

                    let unique_name = {
                        let mut map = name_counter.lock().expect("Failed to lock mutex");
                        let count = map.entry(name.to_string()).or_insert(0);
                        *count += 1;
                        let current_count = *count;

                        if current_count == 1 {
                            name.to_string()
                        } else {
                            let stem = Path::new(name)
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("file");
                            let ext = Path::new(name)
                                .extension()
                                .and_then(|e| e.to_str())
                                .unwrap_or("");
                            if ext.is_empty() {
                                format!("{}__{}", stem, current_count - 1)
                            } else {
                                format!("{}__{}.{}", stem, current_count - 1, ext)
                            }
                        }
                    };

                    target.join(unique_name)
                }
                CleanMode::Clean | CleanMode::Scan => {
                    target.join(rel)
                }
            };

            if let Some(parent) = dest.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    return (item.clone(), Err(format!("Failed to create directory: {}", e)));
                }
            }

            match fs::copy(&item, &dest) {
                Ok(size) => (item.clone(), Ok(size)),
                Err(e) => (item.clone(), Err(format!("Failed to copy: {}", e))),
            }
        })
        .collect();

    let mut last_emit = std::time::Instant::now();

    for (original_path, result) in results {
        match result {
            Ok(size) => {
                copied += 1;
                bytes_copied += size;

                if last_emit.elapsed().as_millis() > 50 || copied == total_files {
                    last_emit = std::time::Instant::now();
                    let percentage = if total_files > 0 {
                        ((copied as f64 / total_files as f64) * 100.0) as u8
                    } else {
                        100
                    };

                    let current_file = original_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();

                    send_progress(
                        app_handle,
                        "copying",
                        copied,
                        total_files,
                        &current_file,
                        percentage,
                        bytes_copied,
                        0,
                    );
                }
            }
            Err(e) => {
                skipped += 1;
                log::warn!("Skipped file {:?}: {}", original_path, e);
            }
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

/// Stub for future AI text analysis feature.
pub async fn analyze_text_with_ai(_text: &str, _output_path: &Path) -> Result<(), String> {
    Err("AI analysis feature is not yet available".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

use std::process::Command;
use std::sync::OnceLock;

static RESOLVED_PATH: OnceLock<String> = OnceLock::new();

#[cfg(windows)]
fn path_sep() -> char {
    ';'
}

#[cfg(not(windows))]
fn path_sep() -> char {
    ':'
}

fn path_entry_count(path: &str) -> usize {
    path
        .split(path_sep())
        .filter(|s| !s.trim().is_empty())
        .count()
}

/// Returns a PATH string that mirrors the user's login shell (Unix) or
/// machine + user PATH from the registry / PowerShell (Windows).
///
/// macOS `.app` bundles launched from Finder inherit a minimal PATH
/// (`/usr/bin:/bin:/usr/sbin:/sbin`). GUI apps on Windows similarly get a
/// reduced PATH. This function resolves a fuller PATH once and caches it.
pub fn full_path() -> &'static str {
    RESOLVED_PATH.get_or_init(|| {
        if let Some(p) = path_from_login_shell() {
            log::info!(
                "[shell_env] resolved PATH from login shell ({} entries)",
                path_entry_count(&p)
            );
            return p;
        }
        let enriched = enriched_fallback();
        log::info!(
            "[shell_env] using fallback PATH ({} entries)",
            path_entry_count(&enriched)
        );
        enriched
    })
}

#[cfg(not(windows))]
fn path_from_login_shell() -> Option<String> {
    let shells = ["/bin/zsh", "/bin/bash", "/bin/sh"];
    for shell in shells {
        if let Ok(output) = Command::new(shell)
            .args(["-l", "-i", "-c", "echo \"__PATH_START__${PATH}__PATH_END__\""])
            .env("TERM", "dumb")
            .stdin(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(start) = stdout.find("__PATH_START__") {
                    if let Some(end) = stdout.find("__PATH_END__") {
                        let path = &stdout[start + 14..end];
                        if !path.is_empty() && path.contains('/') {
                            return Some(path.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

#[cfg(windows)]
fn path_from_login_shell() -> Option<String> {
    // Merge Machine + User PATH the same way Windows does for interactive sessions.
    let ps = r#"Write-Output ("__PATH_START__" + ([Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User')) + "__PATH_END__")"#;
    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", ps])
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let start = stdout.find("__PATH_START__")?;
    let end = stdout.find("__PATH_END__")?;
    let path = &stdout[start + 14..end];
    if path.is_empty() {
        return None;
    }
    // Windows PATH uses semicolons and/or drive letters
    if path.contains(';') || path.contains('\\') {
        return Some(path.to_string());
    }
    None
}

#[cfg(not(windows))]
fn enriched_fallback() -> String {
    let current = std::env::var("PATH").unwrap_or_default();
    let home = dirs::home_dir()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let extra_dirs = [
        "/usr/local/bin".to_string(),
        "/opt/homebrew/bin".to_string(),
        "/opt/homebrew/sbin".to_string(),
        format!("{}/.local/bin", home),
        format!("{}/.cargo/bin", home),
        format!("{}/go/bin", home),
        "/usr/local/go/bin".to_string(),
        format!("{}/.nvm/versions/node", home),
    ];

    let mut parts: Vec<String> = current.split(':').map(String::from).collect();

    if let Ok(nvm_dir) = glob_latest_nvm_bin(&home) {
        if !parts.contains(&nvm_dir) {
            parts.push(nvm_dir);
        }
    }

    for dir in &extra_dirs {
        if !parts.iter().any(|p| p == dir) && std::path::Path::new(dir).is_dir() {
            parts.push(dir.clone());
        }
    }

    parts.join(":")
}

#[cfg(windows)]
fn enriched_fallback() -> String {
    let current = std::env::var("PATH").unwrap_or_default();
    let home = dirs::home_dir()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let program_files = std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string());
    let program_files_x86 = std::env::var("ProgramFiles(x86)")
        .unwrap_or_else(|_| "C:\\Program Files (x86)".to_string());

    let extra_dirs = vec![
        format!(r"{}\.cargo\bin", home),
        format!(r"{program_files}\Git\bin"),
        format!(r"{program_files}\Git\cmd"),
        format!(r"{program_files}\Git\usr\bin"),
        format!(r"{program_files_x86}\Git\bin"),
        format!(r"{program_files}\nodejs"),
        format!(r"{home}\AppData\Roaming\npm"),
        std::env::var("LOCALAPPDATA")
            .map(|l| format!(r"{l}\Programs"))
            .unwrap_or_default(),
    ];

    let mut parts: Vec<String> = current
        .split(';')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    for dir in extra_dirs {
        if dir.is_empty() {
            continue;
        }
        if !parts.iter().any(|p| p == &dir) && std::path::Path::new(&dir).is_dir() {
            parts.push(dir);
        }
    }

    parts.join(";")
}

#[cfg(not(windows))]
fn glob_latest_nvm_bin(home: &str) -> Result<String, ()> {
    let nvm_node = format!("{}/.nvm/versions/node", home);
    let nvm_path = std::path::Path::new(&nvm_node);
    if !nvm_path.is_dir() {
        return Err(());
    }
    let mut versions: Vec<_> = std::fs::read_dir(nvm_path)
        .map_err(|_| ())?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .collect();
    versions.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    versions
        .first()
        .map(|v| v.path().join("bin").to_string_lossy().to_string())
        .ok_or(())
}

/// Apply the resolved PATH to a `std::process::Command`.
pub fn apply_env(cmd: &mut Command) -> &mut Command {
    cmd.env("PATH", full_path())
}

/// Apply the resolved PATH to a `tokio::process::Command`.
pub fn apply_env_async(cmd: &mut tokio::process::Command) -> &mut tokio::process::Command {
    cmd.env("PATH", full_path())
}

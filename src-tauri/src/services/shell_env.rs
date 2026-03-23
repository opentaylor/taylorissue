use std::process::Command;
use std::sync::RwLock;

static RESOLVED_PATH: RwLock<Option<String>> = RwLock::new(None);

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

fn resolve_path() -> String {
    let base = match path_from_login_shell() {
        Some(p) => {
            log::info!(
                "[shell_env] resolved PATH from login shell ({} entries)",
                path_entry_count(&p)
            );
            p
        }
        None => {
            log::info!("[shell_env] login shell resolution failed, using process PATH");
            std::env::var("PATH").unwrap_or_default()
        }
    };
    let merged = merge_extra_paths(&base);
    log::info!(
        "[shell_env] final PATH ({} entries)",
        path_entry_count(&merged)
    );
    merged
}

pub fn full_path() -> String {
    {
        let guard = RESOLVED_PATH.read().unwrap();
        if let Some(ref path) = *guard {
            return path.clone();
        }
    }
    let mut guard = RESOLVED_PATH.write().unwrap();
    if let Some(ref path) = *guard {
        return path.clone();
    }
    let path = resolve_path();
    *guard = Some(path.clone());
    path
}

pub fn refresh_path() {
    let path = resolve_path();
    log::info!(
        "[shell_env] PATH refreshed ({} entries)",
        path_entry_count(&path)
    );
    let mut guard = RESOLVED_PATH.write().unwrap();
    *guard = Some(path);
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
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let ps = r#"Write-Output ("__PATH_START__" + ([Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User')) + "__PATH_END__")"#;
    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", ps])
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
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

    if path.contains(';') || path.contains('\\') {
        return Some(path.to_string());
    }
    None
}

#[cfg(not(windows))]
fn merge_extra_paths(base: &str) -> String {
    let home = dirs::home_dir()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let priority_dirs = [
        format!("{}/.taylorissue/app/node_modules/.bin", home),
    ];

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

    let mut parts: Vec<String> = Vec::new();

    for dir in &priority_dirs {
        if std::path::Path::new(dir).is_dir() {
            parts.push(dir.clone());
        }
    }

    for entry in base.split(':') {
        let entry = entry.to_string();
        if !entry.is_empty() && !parts.iter().any(|p| p == &entry) {
            parts.push(entry);
        }
    }

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
fn merge_extra_paths(base: &str) -> String {
    let home = dirs::home_dir()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let local_app_data = std::env::var("LOCALAPPDATA")
        .unwrap_or_else(|_| format!(r"{home}\AppData\Local"));

    let program_files = std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string());
    let program_files_x86 = std::env::var("ProgramFiles(x86)")
        .unwrap_or_else(|_| "C:\\Program Files (x86)".to_string());

    let priority_dirs = vec![
        format!(r"{local_app_data}\taylorissue\app\node_modules\.bin"),
        format!(r"{local_app_data}\taylorissue\deps\portable-git\cmd"),
        format!(r"{local_app_data}\taylorissue\deps\portable-git\mingw64\bin"),
        format!(r"{local_app_data}\taylorissue\deps\nodejs"),
    ];

    let extra_dirs = vec![
        format!(r"{}\.cargo\bin", home),
        format!(r"{program_files}\Git\bin"),
        format!(r"{program_files}\Git\cmd"),
        format!(r"{program_files}\Git\usr\bin"),
        format!(r"{program_files_x86}\Git\bin"),
        format!(r"{program_files}\nodejs"),
        format!(r"{home}\AppData\Roaming\npm"),
        format!(r"{local_app_data}\Programs"),
    ];

    let mut parts: Vec<String> = Vec::new();

    for dir in &priority_dirs {
        if !dir.is_empty() && std::path::Path::new(dir).is_dir() {
            parts.push(dir.clone());
        }
    }

    for entry in base.split(';') {
        let entry = entry.trim().to_string();
        if !entry.is_empty() && !parts.iter().any(|p| p == &entry) {
            parts.push(entry);
        }
    }

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

pub fn build_command(bin: &str) -> Command {
    #[cfg(windows)]
    {
        let lower = bin.to_lowercase();
        if lower.ends_with(".cmd") || lower.ends_with(".bat") {
            let mut cmd = Command::new("cmd.exe");
            cmd.args(["/c", bin]);
            return cmd;
        }
    }
    Command::new(bin)
}

pub fn apply_env(cmd: &mut Command) -> &mut Command {
    cmd.env("PATH", full_path())
}

pub fn apply_env_async(cmd: &mut tokio::process::Command) -> &mut tokio::process::Command {
    cmd.env("PATH", full_path())
}

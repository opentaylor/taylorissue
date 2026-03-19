use std::path::PathBuf;

#[cfg(not(windows))]
const ARCHIVE_URL: &str =
    "https://github.com/openclaw/openclaw/archive/refs/heads/main.tar.gz";

#[cfg(windows)]
const ARCHIVE_URL_ZIP: &str =
    "https://github.com/openclaw/openclaw/archive/refs/heads/main.zip";

/// Returns the platform-specific taylorissue data directory.
fn taylorissue_dir() -> PathBuf {
    #[cfg(windows)]
    {
        std::env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .unwrap_or_default()
                    .join("AppData")
                    .join("Local")
            })
            .join("taylorissue")
    }
    #[cfg(not(windows))]
    {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".taylorissue")
    }
}

fn openclaw_ref_dir() -> PathBuf {
    taylorissue_dir().join("openclaw")
}

/// Ensures the OpenClaw source reference is available on disk.
/// Downloads from GitHub on first call. Returns the path if available.
pub async fn ensure_openclaw_ref() -> Option<String> {
    let ref_dir = openclaw_ref_dir();

    if ref_dir.join("package.json").exists() {
        log::info!("[openclaw_ref] reference already present at {}", ref_dir.display());
        return Some(ref_dir.to_string_lossy().to_string());
    }

    log::info!("[openclaw_ref] downloading OpenClaw source reference...");
    match download_ref(&ref_dir).await {
        Ok(()) => {
            log::info!("[openclaw_ref] reference ready at {}", ref_dir.display());
            Some(ref_dir.to_string_lossy().to_string())
        }
        Err(e) => {
            log::error!("[openclaw_ref] download failed: {e}");
            None
        }
    }
}

async fn download_ref(dest: &std::path::Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let parent = dest.parent().ok_or("no parent directory")?;
    tokio::fs::create_dir_all(parent).await?;

    #[cfg(not(windows))]
    {
        download_ref_unix(dest, parent).await
    }
    #[cfg(windows)]
    {
        download_ref_windows(dest, parent).await
    }
}

#[cfg(not(windows))]
async fn download_ref_unix(
    dest: &std::path::Path,
    parent: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let tmp_tar = std::env::temp_dir().join("openclaw-ref.tar.gz");

    let dl = tokio::process::Command::new("curl")
        .args(["-fSL", "-o"])
        .arg(&tmp_tar)
        .arg(ARCHIVE_URL)
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .output()
        .await?;
    if !dl.status.success() {
        let stderr = String::from_utf8_lossy(&dl.stderr);
        return Err(format!("curl failed: {stderr}").into());
    }

    let ex = tokio::process::Command::new("tar")
        .args(["-xzf"])
        .arg(&tmp_tar)
        .arg("-C")
        .arg(parent)
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .output()
        .await?;
    if !ex.status.success() {
        let stderr = String::from_utf8_lossy(&ex.stderr);
        return Err(format!("tar failed: {stderr}").into());
    }

    let extracted = parent.join("openclaw-main");
    if extracted.exists() {
        if dest.exists() {
            tokio::fs::remove_dir_all(dest).await?;
        }
        tokio::fs::rename(&extracted, dest).await?;
    }

    let _ = tokio::fs::remove_file(&tmp_tar).await;
    Ok(())
}

#[cfg(windows)]
async fn download_ref_windows(
    dest: &std::path::Path,
    parent: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let tmp_zip = std::env::temp_dir().join("openclaw-ref.zip");

    let ps_download = format!(
        "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; \
         Invoke-WebRequest -Uri '{}' -OutFile '{}' -UseBasicParsing",
        ARCHIVE_URL_ZIP,
        tmp_zip.display()
    );
    let dl = tokio::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps_download])
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .await?;
    if !dl.status.success() {
        let stderr = String::from_utf8_lossy(&dl.stderr);
        return Err(format!("download failed: {stderr}").into());
    }

    let ps_extract = format!(
        "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
        tmp_zip.display(),
        parent.display()
    );
    let ex = tokio::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps_extract])
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .await?;
    if !ex.status.success() {
        let stderr = String::from_utf8_lossy(&ex.stderr);
        return Err(format!("extract failed: {stderr}").into());
    }

    let extracted = parent.join("openclaw-main");
    if extracted.exists() {
        if dest.exists() {
            tokio::fs::remove_dir_all(dest).await?;
        }
        tokio::fs::rename(&extracted, dest).await?;
    }

    let _ = tokio::fs::remove_file(&tmp_zip).await;
    Ok(())
}

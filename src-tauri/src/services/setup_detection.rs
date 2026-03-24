use std::path::PathBuf;
use serde_json::Value;

use crate::services::shell_env;

#[derive(Debug, Clone, PartialEq)]
pub enum InstallType {
    Official,
    TaylorIssue,
}

impl InstallType {
    pub fn as_str(&self) -> &str {
        match self {
            InstallType::Official => "official",
            InstallType::TaylorIssue => "taylorissue",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedOpenClaw {
    pub bin_path: String,
    pub install_type: InstallType,
}

fn openclaw_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_default().join(".openclaw")
}

fn openclaw_config_path() -> PathBuf {
    openclaw_dir().join("openclaw.json")
}

fn read_openclaw_config() -> Option<Value> {
    let path = openclaw_config_path();
    let content = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn detect_workspace_path() -> Option<String> {
    let current = openclaw_dir().join("agents").join("main").join("workspace");
    if current.is_dir() {
        return Some(current.to_string_lossy().to_string());
    }
    let legacy = openclaw_dir().join("workspace");
    if legacy.is_dir() {
        return Some(legacy.to_string_lossy().to_string());
    }
    None
}

fn is_taylorissue_path(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.contains("taylorissue") || lower.contains(".taylorissue")
}

fn taylorissue_openclaw_path() -> PathBuf {
    #[cfg(windows)]
    {
        let local = std::env::var("LOCALAPPDATA").unwrap_or_default();
        PathBuf::from(local).join(r"taylorissue\app\node_modules\.bin\openclaw.cmd")
    }
    #[cfg(not(windows))]
    {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".taylorissue/app/node_modules/.bin/openclaw")
    }
}

fn exclude_taylorissue_dirs(path: &str) -> String {
    let sep = if cfg!(windows) { ';' } else { ':' };
    path.split(sep)
        .filter(|dir| !dir.trim().is_empty() && !is_taylorissue_path(dir))
        .collect::<Vec<_>>()
        .join(&sep.to_string())
}

pub fn detect_openclaw_bin() -> Option<ResolvedOpenClaw> {
    let enriched = shell_env::full_path();
    let system_path = exclude_taylorissue_dirs(&enriched);

    if let Ok(p) = which::which_in("openclaw", Some(&system_path), ".") {
        let path_str = p.to_string_lossy().to_string();
        log::info!("[detect] found official openclaw at {}", path_str);
        return Some(ResolvedOpenClaw {
            bin_path: path_str,
            install_type: InstallType::Official,
        });
    }

    let ti_path = taylorissue_openclaw_path();
    if ti_path.is_file() {
        let path_str = ti_path.to_string_lossy().to_string();
        log::info!("[detect] found taylorissue openclaw at {}", path_str);
        return Some(ResolvedOpenClaw {
            bin_path: path_str,
            install_type: InstallType::TaylorIssue,
        });
    }

    log::info!("[detect] no openclaw binary found");
    None
}

pub fn detect_openclaw_bin_path() -> Option<String> {
    detect_openclaw_bin().map(|r| r.bin_path)
}

pub fn detect_gateway_token() -> Option<String> {
    let cfg = read_openclaw_config()?;
    cfg.get("gateway")?
        .get("auth")?
        .get("token")?
        .as_str()
        .map(String::from)
}

pub fn detect_gateway_port() -> Option<u16> {
    let cfg = read_openclaw_config()?;
    let gw = cfg.get("gateway")?;
    if let Some(port) = gw.get("http").and_then(|h| h.get("port")).and_then(|v| v.as_u64()) {
        return Some(port as u16);
    }
    gw.get("port").and_then(|v| v.as_u64()).map(|v| v as u16)
}

pub fn check_http_endpoint_enabled() -> Option<bool> {
    let cfg = read_openclaw_config()?;
    let enabled = cfg
        .get("gateway")?
        .get("http")?
        .get("endpoints")?
        .get("chatCompletions")?
        .get("enabled")?
        .as_bool();
    Some(enabled.unwrap_or(false))
}

fn ensure_dict<'a>(parent: &'a mut Value, key: &str) -> &'a mut Value {
    if !parent.get(key).map(|v| v.is_object()).unwrap_or(false) {
        parent[key] = serde_json::json!({});
    }
    parent.get_mut(key).unwrap()
}

pub fn enable_http_endpoint() -> bool {
    let path = openclaw_config_path();
    if !path.is_file() {
        return false;
    }
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let mut cfg: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return false,
    };
    if !cfg.is_object() {
        return false;
    }

    let gw = ensure_dict(&mut cfg, "gateway");
    let http = ensure_dict(gw, "http");
    let endpoints = ensure_dict(http, "endpoints");
    let cc = ensure_dict(endpoints, "chatCompletions");

    if cc.get("enabled").and_then(|v: &Value| v.as_bool()) == Some(true) {
        return true;
    }
    cc["enabled"] = Value::Bool(true);

    let output = serde_json::to_string_pretty(&cfg).unwrap_or_default();
    std::fs::write(&path, format!("{}\n", output)).is_ok()
}

pub struct DetectionResult {
    pub workspace_path: Option<String>,
    pub openclaw_bin: Option<String>,
    pub openclaw_install_type: Option<String>,
    pub gateway_token: Option<String>,
    pub gateway_port: Option<u16>,
    pub http_endpoint_enabled: Option<bool>,
}

pub fn detect_all() -> DetectionResult {
    let resolved = detect_openclaw_bin();
    DetectionResult {
        workspace_path: detect_workspace_path(),
        openclaw_bin: resolved.as_ref().map(|r| r.bin_path.clone()),
        openclaw_install_type: resolved.as_ref().map(|r| r.install_type.as_str().to_string()),
        gateway_token: detect_gateway_token(),
        gateway_port: detect_gateway_port(),
        http_endpoint_enabled: check_http_endpoint_enabled(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openclaw_dir() {
        let dir = openclaw_dir();
        assert!(dir.to_string_lossy().contains(".openclaw"));
    }

    #[test]
    fn test_detect_workspace_path_returns_option() {
        let _ = detect_workspace_path();
    }

    #[test]
    fn test_detect_all_runs() {
        let result = detect_all();

        let _ = result.workspace_path;
    }
}

use std::path::PathBuf;
use serde_json::Value;

use crate::services::shell_env;

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

pub fn detect_openclaw_bin() -> Option<String> {
    let enriched = shell_env::full_path();
    if let Ok(p) = which::which_in("openclaw", Some(&enriched), ".") {
        return Some(p.to_string_lossy().to_string());
    }

    if let Ok(p) = which::which("openclaw") {
        return Some(p.to_string_lossy().to_string());
    }

    #[cfg(windows)]
    {
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            let cmd_path = PathBuf::from(&local)
                .join(r"taylorissue\app\node_modules\.bin\openclaw.cmd");
            if cmd_path.is_file() {
                return Some(cmd_path.to_string_lossy().to_string());
            }
        }
    }

    None
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
    pub gateway_token: Option<String>,
    pub gateway_port: Option<u16>,
    pub http_endpoint_enabled: Option<bool>,
}

pub fn detect_all() -> DetectionResult {
    DetectionResult {
        workspace_path: detect_workspace_path(),
        openclaw_bin: detect_openclaw_bin(),
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

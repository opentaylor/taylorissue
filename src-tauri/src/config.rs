use serde::{Deserialize, Serialize};

use crate::services::setup_detection;

fn default_provider() -> String {
    "openai".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_provider")]
    pub provider: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub workspace_path: String,
    #[serde(default)]
    pub gateway_token: String,
    #[serde(default)]
    pub gateway_port: u16,
    #[serde(default)]
    pub openclaw_bin: String,
    #[serde(default)]
    pub openclaw_install_type: String,
}

impl AppConfig {
    pub fn resolved(mut self) -> Self {
        self.workspace_path = self.resolve_workspace_path();

        if self.gateway_token.is_empty() {
            self.gateway_token = setup_detection::detect_gateway_token()
                .unwrap_or_default();
        }
        if self.gateway_port == 0 {
            self.gateway_port = setup_detection::detect_gateway_port()
                .unwrap_or(18789);
        }
        if self.openclaw_bin.is_empty() {
            if let Some(resolved) = setup_detection::detect_openclaw_bin() {
                self.openclaw_bin = resolved.bin_path;
                self.openclaw_install_type = resolved.install_type.as_str().to_string();
            }
        }

        log::info!(
            "[AppConfig::resolved] workspace={} gateway={}:{} token_len={} bin={} type={}",
            self.workspace_path,
            self.gateway_url(),
            self.gateway_port,
            self.gateway_token.len(),
            self.openclaw_bin,
            self.openclaw_install_type,
        );

        self
    }

    pub fn gateway_url(&self) -> String {
        format!("http://localhost:{}/v1", self.gateway_port)
    }

    pub fn resolve_workspace_path(&self) -> String {
        let raw = self.expand_tilde(&self.workspace_path);

        if raw.is_empty() {
            return setup_detection::detect_workspace_path()
                .unwrap_or_else(|| self.expand_tilde("~/.openclaw/workspace"));
        }

        let p = std::path::Path::new(&raw);
        let is_openclaw_dir = p
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n == ".openclaw")
            .unwrap_or(false);

        if is_openclaw_dir {
            let modern = p.join("agents").join("main").join("workspace");
            if modern.is_dir() {
                return modern.to_string_lossy().to_string();
            }
            let legacy = p.join("workspace");
            if legacy.is_dir() {
                return legacy.to_string_lossy().to_string();
            }
            return setup_detection::detect_workspace_path()
                .unwrap_or_else(|| modern.to_string_lossy().to_string());
        }

        raw
    }

    fn expand_tilde(&self, path: &str) -> String {
        if path.starts_with("~/") || path.starts_with("~\\") {
            if let Some(home) = dirs::home_dir() {
                return home.join(&path[2..]).to_string_lossy().to_string();
            }
        }
        if path == "~" {
            if let Some(home) = dirs::home_dir() {
                return home.to_string_lossy().to_string();
            }
        }
        path.to_string()
    }
}

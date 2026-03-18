use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;

use crate::config::AppConfig;
use crate::services::uninstall_runner;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UninstallRequest {
    pub config: AppConfig,
    pub selected_options: Vec<String>,
}

#[tauri::command]
pub async fn start_uninstall(
    request: UninstallRequest,
    on_event: Channel<serde_json::Value>,
) -> Result<(), String> {
    uninstall_runner::run_uninstall(request.config.resolved(), request.selected_options, on_event)
        .await
        .map_err(|e| e.to_string())
}

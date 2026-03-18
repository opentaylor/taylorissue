use tauri::ipc::Channel;
use tauri::AppHandle;

use crate::config::AppConfig;
use crate::services::install_runner;

#[tauri::command]
pub async fn start_install(
    app: AppHandle,
    config: AppConfig,
    on_event: Channel<serde_json::Value>,
) -> Result<(), String> {
    install_runner::run_install(app, config.resolved(), on_event)
        .await
        .map_err(|e| e.to_string())
}

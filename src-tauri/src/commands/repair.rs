use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;

use crate::config::AppConfig;
use crate::services::repair_runner;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixRequest {
    pub config: AppConfig,
    pub session_id: String,
    pub step_id: String,
    pub issue_description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomFixRequest {
    pub config: AppConfig,
    pub problem: String,
}

#[tauri::command]
pub async fn start_repair(
    config: AppConfig,
    on_event: Channel<serde_json::Value>,
) -> Result<(), String> {
    repair_runner::run_repair(config.resolved(), on_event)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fix_step(
    request: FixRequest,
    on_event: Channel<serde_json::Value>,
) -> Result<(), String> {
    repair_runner::run_fix(
        request.config.resolved(),
        &request.session_id,
        &request.step_id,
        &request.issue_description,
        on_event,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_custom_fix(
    request: CustomFixRequest,
    on_event: Channel<serde_json::Value>,
) -> Result<(), String> {
    repair_runner::run_custom_fix(request.config.resolved(), &request.problem, on_event)
        .await
        .map_err(|e| e.to_string())
}

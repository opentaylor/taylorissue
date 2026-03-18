use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;

use crate::config::AppConfig;
use crate::services::{conversation_store, message_service};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub config: AppConfig,
    pub messages: Vec<ChatMessage>,
    pub operator_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppendMessagesRequest {
    pub messages: Vec<conversation_store::StoredMessage>,
}

#[tauri::command]
pub async fn list_agents(config: AppConfig) -> Result<serde_json::Value, String> {
    let config = config.resolved();
    let agents = message_service::list_registered_agents(&config)
        .map_err(|e| e.to_string())?;
    serde_json::to_value(&agents).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn message_chat(
    agent_id: String,
    request: ChatRequest,
    on_event: Channel<serde_json::Value>,
) -> Result<(), String> {
    let config = request.config.resolved();
    let messages: Vec<(String, String)> = request
        .messages
        .into_iter()
        .map(|m| (m.role, m.content))
        .collect();
    let operator_name = request.operator_name.unwrap_or_else(|| "Operator".to_string());
    message_service::stream_chat(&agent_id, messages, &operator_name, &config, on_event)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_conversation(agent_id: String, config: AppConfig) -> Result<serde_json::Value, String> {
    let config = config.resolved();
    conversation_store::validate_agent_id(&agent_id).map_err(|e| e.to_string())?;
    let workspace = config.resolve_workspace_path();
    let messages = conversation_store::get_messages(&workspace, &agent_id)
        .map_err(|e| e.to_string())?;
    serde_json::to_value(&messages).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn append_conversation(
    agent_id: String,
    request: AppendMessagesRequest,
    config: AppConfig,
) -> Result<serde_json::Value, String> {
    let config = config.resolved();
    conversation_store::validate_agent_id(&agent_id).map_err(|e| e.to_string())?;
    let workspace = config.resolve_workspace_path();
    conversation_store::append_messages(&workspace, &agent_id, &request.messages)
        .map_err(|e| e.to_string())?;
    Ok(serde_json::json!({"ok": true}))
}

#[tauri::command]
pub async fn clear_conversation(agent_id: String, config: AppConfig) -> Result<serde_json::Value, String> {
    let config = config.resolved();
    conversation_store::validate_agent_id(&agent_id).map_err(|e| e.to_string())?;
    let workspace = config.resolve_workspace_path();
    conversation_store::clear_conversation(&workspace, &agent_id)
        .map_err(|e| e.to_string())?;
    Ok(serde_json::json!({"ok": true}))
}

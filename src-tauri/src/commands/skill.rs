use crate::config::AppConfig;
use crate::services::{clawhub_client, skill_store};

#[tauri::command]
pub async fn list_skills(config: AppConfig) -> Result<serde_json::Value, String> {
    let config = config.resolved();
    let skills = skill_store::list_skills(&config.openclaw_bin)
        .await
        .map_err(|e| e.to_string())?;
    serde_json::to_value(&skills).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn install_skill(name: String, config: AppConfig) -> Result<serde_json::Value, String> {
    let config = config.resolved();
    let result = skill_store::install_skill(&config, &name)
        .await
        .map_err(|e| e.to_string())?;
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn uninstall_skill(name: String, config: AppConfig) -> Result<serde_json::Value, String> {
    let config = config.resolved();
    skill_store::uninstall_skill(&config.openclaw_bin, &name)
        .await
        .map_err(|e| e.to_string())?;
    Ok(serde_json::json!({"ok": true}))
}

#[tauri::command]
pub async fn search_clawhub(query: String) -> Result<serde_json::Value, String> {
    let results = clawhub_client::search_skills(&query)
        .await
        .map_err(|e| e.to_string())?;
    serde_json::to_value(&results).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn install_clawhub_skill(slug: String, config: AppConfig) -> Result<serde_json::Value, String> {
    let config = config.resolved();
    let result = skill_store::install_clawhub_skill(&config, &slug).await;
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

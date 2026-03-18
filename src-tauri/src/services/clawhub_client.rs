use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::Command;
use thiserror::Error;

use crate::services::shell_env;

const CLAWHUB_BASE: &str = "https://clawhub.ai";
const TIMEOUT_SECS: u64 = 15;

#[derive(Error, Debug)]
pub enum ClawHubError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Install failed: {0}")]
    InstallFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClawHubSkill {
    pub slug: String,
    pub name: String,
    pub summary: String,
    pub version: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClawHubInstallResult {
    pub ok: bool,
    pub outputs: Vec<String>,
}

pub async fn search_skills(query: &str) -> Result<Vec<ClawHubSkill>, ClawHubError> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(TIMEOUT_SECS))
        .build()?;

    let resp = client
        .get(format!("{}/api/v1/search", CLAWHUB_BASE))
        .query(&[("q", query)])
        .send()
        .await?;

    let data: Value = resp.json().await?;
    let results = data
        .get("results")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    Ok(results.into_iter().map(|r| normalize_search_result(&r)).collect())
}

pub async fn get_skill(slug: &str) -> Result<Option<Value>, ClawHubError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(TIMEOUT_SECS))
        .build()?;

    let resp = client
        .get(format!("{}/api/v1/skills/{}", CLAWHUB_BASE, slug))
        .send()
        .await?;

    if resp.status().as_u16() == 404 {
        return Ok(None);
    }
    let data: Value = resp.json().await?;
    Ok(Some(data))
}

pub async fn install_skill_cli(slug: &str) -> Result<ClawHubInstallResult, ClawHubError> {
    let slug = slug.to_string();
    tokio::task::spawn_blocking(move || install_skill_cli_sync(&slug))
        .await
        .unwrap_or_else(|_| {
            Ok(ClawHubInstallResult {
                ok: false,
                outputs: vec!["Task panicked".to_string()],
            })
        })
}

fn install_skill_cli_sync(slug: &str) -> Result<ClawHubInstallResult, ClawHubError> {
    let (cmd_name, cmd_args) = if which::which("clawhub").is_ok() {
        ("clawhub".to_string(), vec!["install".to_string(), "--force".to_string(), slug.to_string()])
    } else {
        let npx = which::which("npx")
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "npx".to_string());
        (npx, vec!["-y".to_string(), "clawhub".to_string(), "install".to_string(), "--force".to_string(), slug.to_string()])
    };

    let mut outputs = vec![format!("Running: {} {}", cmd_name, cmd_args.join(" "))];

    let mut cmd = Command::new(&cmd_name);
    cmd.args(&cmd_args);
    shell_env::apply_env(&mut cmd);
    let result = cmd.output();

    match result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if !stdout.is_empty() {
                outputs.push(stdout);
            }
            if !stderr.is_empty() {
                outputs.push(stderr);
            }
            if output.status.success() {
                outputs.push(format!("OK: {} installed successfully", slug));
                Ok(ClawHubInstallResult { ok: true, outputs })
            } else {
                outputs.push(format!("FAILED: exit code {:?}", output.status.code()));
                Ok(ClawHubInstallResult { ok: false, outputs })
            }
        }
        Err(_) => {
            outputs.push("FAILED: neither 'clawhub' nor 'npx' found on PATH".to_string());
            Ok(ClawHubInstallResult { ok: false, outputs })
        }
    }
}

fn normalize_search_result(r: &Value) -> ClawHubSkill {
    ClawHubSkill {
        slug: r.get("slug").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        name: r
            .get("displayName")
            .or(r.get("slug"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        summary: r.get("summary").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        version: r.get("version").and_then(|v| v.as_str()).map(String::from),
        updated_at: r.get("updatedAt").and_then(|v| v.as_str()).map(String::from),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_search_result() {
        let r = serde_json::json!({
            "slug": "test-skill",
            "displayName": "Test Skill",
            "summary": "A test skill",
            "version": "1.0.0"
        });
        let skill = normalize_search_result(&r);
        assert_eq!(skill.slug, "test-skill");
        assert_eq!(skill.name, "Test Skill");
    }

    #[tokio::test]
    async fn test_search_empty_query() {
        let result = search_skills("").await.unwrap();
        assert!(result.is_empty());
    }
}

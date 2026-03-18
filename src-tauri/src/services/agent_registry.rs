use serde::{Deserialize, Serialize};
use std::path::PathBuf;

static COLORS: &[&str] = &[
    "#f5c518", "#a855f7", "#3b82f6", "#22c55e", "#f97316",
    "#eab308", "#14b8a6", "#60a5fa", "#f59e0b", "#94a3b8",
    "#06b6d4", "#ec4899", "#84cc16", "#8b5cf6", "#ef4444",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEntry {
    pub id: String,
    pub name: String,
    pub title: String,
    pub reports_to: Option<String>,
    pub direct_reports: Vec<String>,
    pub soul_path: Option<String>,
    pub color: String,
    pub emoji: String,
    pub model: Option<String>,
    pub description: String,
}

fn agents_root(workspace_path: &str) -> PathBuf {
    let ws = PathBuf::from(workspace_path);
    let candidate = ws.parent().and_then(|p| p.parent()).unwrap_or(&ws);
    if candidate.file_name().and_then(|n| n.to_str()) == Some("agents") {
        return candidate.to_path_buf();
    }
    let mut current = ws.as_path();
    while let Some(parent) = current.parent() {
        if parent.file_name().and_then(|n| n.to_str()) == Some("agents") {
            if let Some(grandparent) = parent.parent() {
                if grandparent.file_name().and_then(|n| n.to_str()) == Some(".openclaw") {
                    return parent.to_path_buf();
                }
            }
        }
        current = parent;
    }
    dirs::home_dir()
        .unwrap_or_default()
        .join(".openclaw")
        .join("agents")
}

fn slug_to_name(slug: &str) -> String {
    slug.replace('-', " ")
        .replace('_', " ")
        .split_whitespace()
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn load_registry(workspace_path: Option<&str>, _openclaw_bin: Option<&str>) -> Vec<AgentEntry> {
    let workspace_path = match workspace_path {
        Some(wp) if !wp.is_empty() => wp,
        _ => return Vec::new(),
    };

    let agents_dir = agents_root(workspace_path);
    if !agents_dir.is_dir() {
        return Vec::new();
    }

    let mut dirs: Vec<String> = match std::fs::read_dir(&agents_dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect(),
        Err(_) => return Vec::new(),
    };
    dirs.sort();

    dirs.iter()
        .enumerate()
        .map(|(idx, name)| {
            let display_name = slug_to_name(name);
            let emoji = display_name.chars().next().map(|c| c.to_string()).unwrap_or_else(|| "?".to_string());
            AgentEntry {
                id: name.clone(),
                name: display_name,
                title: "Agent".to_string(),
                reports_to: None,
                direct_reports: Vec::new(),
                soul_path: None,
                color: COLORS[idx % COLORS.len()].to_string(),
                emoji,
                model: None,
                description: String::new(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slug_to_name() {
        assert_eq!(slug_to_name("hello-world"), "Hello World");
        assert_eq!(slug_to_name("my_agent"), "My Agent");
        assert_eq!(slug_to_name("main"), "Main");
    }

    #[test]
    fn test_load_registry_empty() {
        let result = load_registry(None, None);
        assert!(result.is_empty());
    }

    #[test]
    fn test_load_registry_empty_string() {
        let result = load_registry(Some(""), None);
        assert!(result.is_empty());
    }

    #[test]
    fn test_load_registry_with_temp_dir() {
        let dir = tempfile::tempdir().unwrap();
        let agents_dir = dir.path().join(".openclaw").join("agents");
        std::fs::create_dir_all(agents_dir.join("agent-a")).unwrap();
        std::fs::create_dir_all(agents_dir.join("agent-b")).unwrap();
        let ws = agents_dir.join("agent-a").join("workspace");
        std::fs::create_dir_all(&ws).unwrap();

        let result = load_registry(Some(ws.to_str().unwrap()), None);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "Agent A");
        assert_eq!(result[1].name, "Agent B");
    }
}

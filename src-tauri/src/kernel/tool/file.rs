use async_trait::async_trait;
use serde_json::Value;
use std::fs;
use std::path::Path;

use super::base::BaseTool;

// ---- EditTool ----

pub struct EditTool;

#[async_trait]
impl BaseTool for EditTool {
    fn name(&self) -> &str { "edit" }
    fn description(&self) -> &str { "Replace an exact string in a file with new content." }
    fn params_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {"type": "string", "description": "Path to the file"},
                "old_string": {"type": "string", "description": "Exact text to find"},
                "new_string": {"type": "string", "description": "Replacement text"},
                "replace_all": {"type": "boolean", "description": "Replace all occurrences"}
            },
            "required": ["file_path", "old_string", "new_string"]
        })
    }

    async fn run(&self, args: Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let file_path = args.get("file_path").and_then(|v| v.as_str()).ok_or("Missing file_path")?;
        let old_string = args.get("old_string").and_then(|v| v.as_str()).ok_or("Missing old_string")?;
        let new_string = args.get("new_string").and_then(|v| v.as_str()).ok_or("Missing new_string")?;
        let replace_all = args.get("replace_all").and_then(|v| v.as_bool()).unwrap_or(false);

        let content = fs::read_to_string(file_path)?;

        if !content.contains(old_string) {
            return Ok(format!("Error: '{}' not found in {}", old_string, file_path));
        }

        let new_content = if replace_all {
            content.replace(old_string, new_string)
        } else {
            content.replacen(old_string, new_string, 1)
        };

        fs::write(file_path, &new_content)?;
        Ok(format!("Successfully edited {}", file_path))
    }
}

// ---- FindTool ----

pub struct FindTool;

#[async_trait]
impl BaseTool for FindTool {
    fn name(&self) -> &str { "find" }
    fn description(&self) -> &str { "Find files matching a glob pattern." }
    fn params_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {"type": "string", "description": "Glob pattern"},
                "path": {"type": "string", "description": "Root directory"}
            },
            "required": ["pattern"]
        })
    }

    async fn run(&self, args: Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let pattern = args.get("pattern").and_then(|v| v.as_str()).ok_or("Missing pattern")?;
        let root = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");

        let full_pattern = if pattern.starts_with('/') || pattern.starts_with('.') {
            pattern.to_string()
        } else {
            format!("{}/{}", root, pattern)
        };

        let mut results = Vec::new();
        for entry in glob::glob(&full_pattern)? {
            if let Ok(path) = entry {
                results.push(path.display().to_string());
            }
        }

        if results.is_empty() {
            Ok("No files found.".to_string())
        } else {
            Ok(results.join("\n"))
        }
    }
}

// ---- GrepTool ----

pub struct GrepTool;

#[async_trait]
impl BaseTool for GrepTool {
    fn name(&self) -> &str { "grep" }
    fn description(&self) -> &str { "Search file contents for a regex pattern." }
    fn params_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {"type": "string", "description": "Regex pattern to search for"},
                "path": {"type": "string", "description": "File or directory to search"},
                "include": {"type": "string", "description": "Glob for file filtering"}
            },
            "required": ["pattern"]
        })
    }

    async fn run(&self, args: Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let pattern = args.get("pattern").and_then(|v| v.as_str()).ok_or("Missing pattern")?;
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
        let include = args.get("include").and_then(|v| v.as_str());

        let re = regex::Regex::new(pattern)?;
        let mut results = Vec::new();

        let path = Path::new(path);
        if path.is_file() {
            grep_file(&re, path, &mut results)?;
        } else if path.is_dir() {
            grep_dir(&re, path, include, &mut results)?;
        }

        if results.is_empty() {
            Ok("No matches found.".to_string())
        } else {
            Ok(results.join("\n"))
        }
    }
}

fn grep_file(re: &regex::Regex, path: &Path, results: &mut Vec<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let content = fs::read_to_string(path)?;
    for (i, line) in content.lines().enumerate() {
        if re.is_match(line) {
            results.push(format!("{}:{}:{}", path.display(), i + 1, line));
        }
    }
    Ok(())
}

fn grep_dir(
    re: &regex::Regex,
    dir: &Path,
    include: Option<&str>,
    results: &mut Vec<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let include_pattern = include.map(|p| glob::Pattern::new(p)).transpose()?;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !name.starts_with('.') && name != "node_modules" && name != "__pycache__" {
                grep_dir(re, &path, include, results)?;
            }
        } else if path.is_file() {
            if let Some(ref pat) = include_pattern {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !pat.matches(name) {
                    continue;
                }
            }
            let _ = grep_file(re, &path, results);
        }
    }
    Ok(())
}

// ---- LsTool ----

pub struct LsTool;

#[async_trait]
impl BaseTool for LsTool {
    fn name(&self) -> &str { "ls" }
    fn description(&self) -> &str { "List directory contents." }
    fn params_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "Directory to list"}
            },
            "required": ["path"]
        })
    }

    async fn run(&self, args: Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
        let mut entries = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let meta = entry.metadata()?;
            let name = entry.file_name().to_string_lossy().to_string();
            let suffix = if meta.is_dir() { "/" } else { "" };
            entries.push(format!("{}{}", name, suffix));
        }
        entries.sort();
        Ok(entries.join("\n"))
    }
}

// ---- ReadTool ----

pub struct ReadTool;

#[async_trait]
impl BaseTool for ReadTool {
    fn name(&self) -> &str { "read" }
    fn description(&self) -> &str { "Read a file and return its contents." }
    fn params_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {"type": "string", "description": "Path to the file"},
                "offset": {"type": "integer", "description": "Start line (1-indexed)"},
                "limit": {"type": "integer", "description": "Number of lines to read"}
            },
            "required": ["file_path"]
        })
    }

    async fn run(&self, args: Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let file_path = args.get("file_path").and_then(|v| v.as_str()).ok_or("Missing file_path")?;
        let offset = args.get("offset").and_then(|v| v.as_u64()).map(|v| v as usize);
        let limit = args.get("limit").and_then(|v| v.as_u64()).map(|v| v as usize);

        let content = fs::read_to_string(file_path)?;
        let lines: Vec<&str> = content.lines().collect();

        let start = offset.unwrap_or(1).saturating_sub(1);
        let end = match limit {
            Some(l) => (start + l).min(lines.len()),
            None => lines.len(),
        };

        let selected: Vec<String> = lines[start..end]
            .iter()
            .enumerate()
            .map(|(i, line)| format!("{:>6}|{}", start + i + 1, line))
            .collect();

        Ok(selected.join("\n"))
    }
}

// ---- WriteTool ----

pub struct WriteTool;

#[async_trait]
impl BaseTool for WriteTool {
    fn name(&self) -> &str { "write" }
    fn description(&self) -> &str { "Write content to a file, creating directories if needed." }
    fn params_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {"type": "string", "description": "Path to the file"},
                "content": {"type": "string", "description": "Content to write"}
            },
            "required": ["file_path", "content"]
        })
    }

    async fn run(&self, args: Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let file_path = args.get("file_path").and_then(|v| v.as_str()).ok_or("Missing file_path")?;
        let content = args.get("content").and_then(|v| v.as_str()).ok_or("Missing content")?;

        let path = Path::new(file_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(file_path, content)?;
        Ok(format!("Successfully wrote to {}", file_path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn temp_dir() -> TempDir {
        tempfile::tempdir().unwrap()
    }

    #[tokio::test]
    async fn test_edit_tool() {
        let dir = temp_dir();
        let path = dir.path().join("test.txt");
        fs::write(&path, "hello world").unwrap();

        let tool = EditTool;
        let result = tool.run(serde_json::json!({
            "file_path": path.to_str().unwrap(),
            "old_string": "world",
            "new_string": "rust"
        })).await.unwrap();

        assert!(result.contains("Successfully"));
        assert_eq!(fs::read_to_string(&path).unwrap(), "hello rust");
    }

    #[tokio::test]
    async fn test_edit_tool_not_found() {
        let dir = temp_dir();
        let path = dir.path().join("test.txt");
        fs::write(&path, "hello world").unwrap();

        let tool = EditTool;
        let result = tool.run(serde_json::json!({
            "file_path": path.to_str().unwrap(),
            "old_string": "missing",
            "new_string": "rust"
        })).await.unwrap();

        assert!(result.contains("not found"));
    }

    #[tokio::test]
    async fn test_ls_tool() {
        let dir = temp_dir();
        fs::write(dir.path().join("a.txt"), "").unwrap();
        fs::create_dir(dir.path().join("subdir")).unwrap();

        let tool = LsTool;
        let result = tool.run(serde_json::json!({"path": dir.path().to_str().unwrap()})).await.unwrap();
        assert!(result.contains("a.txt"));
        assert!(result.contains("subdir/"));
    }

    #[tokio::test]
    async fn test_read_tool() {
        let dir = temp_dir();
        let path = dir.path().join("test.txt");
        fs::write(&path, "line1\nline2\nline3").unwrap();

        let tool = ReadTool;
        let result = tool.run(serde_json::json!({"file_path": path.to_str().unwrap()})).await.unwrap();
        assert!(result.contains("line1"));
        assert!(result.contains("line3"));
    }

    #[tokio::test]
    async fn test_read_tool_with_offset_and_limit() {
        let dir = temp_dir();
        let path = dir.path().join("test.txt");
        fs::write(&path, "line1\nline2\nline3\nline4").unwrap();

        let tool = ReadTool;
        let result = tool.run(serde_json::json!({
            "file_path": path.to_str().unwrap(),
            "offset": 2,
            "limit": 2
        })).await.unwrap();
        assert!(result.contains("line2"));
        assert!(result.contains("line3"));
        assert!(!result.contains("line1"));
    }

    #[tokio::test]
    async fn test_write_tool() {
        let dir = temp_dir();
        let path = dir.path().join("subdir/test.txt");

        let tool = WriteTool;
        let result = tool.run(serde_json::json!({
            "file_path": path.to_str().unwrap(),
            "content": "hello rust"
        })).await.unwrap();
        assert!(result.contains("Successfully"));
        assert_eq!(fs::read_to_string(&path).unwrap(), "hello rust");
    }

    #[tokio::test]
    async fn test_grep_tool() {
        let dir = temp_dir();
        fs::write(dir.path().join("test.txt"), "hello\nworld\nhello world").unwrap();

        let tool = GrepTool;
        let result = tool.run(serde_json::json!({
            "pattern": "hello",
            "path": dir.path().join("test.txt").to_str().unwrap()
        })).await.unwrap();
        assert!(result.contains("hello"));
    }

    #[tokio::test]
    async fn test_find_tool() {
        let dir = temp_dir();
        fs::write(dir.path().join("test.txt"), "").unwrap();
        fs::write(dir.path().join("test.rs"), "").unwrap();

        let tool = FindTool;
        let result = tool.run(serde_json::json!({
            "pattern": "*.txt",
            "path": dir.path().to_str().unwrap()
        })).await.unwrap();
        assert!(result.contains("test.txt"));
    }
}

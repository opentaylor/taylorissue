use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::Path;

use super::base::{schema_for, BaseTool};

#[derive(Deserialize, JsonSchema)]
struct EditArgs {
    /// Path to the file
    file_path: String,
    /// Exact text to find
    old_string: String,
    /// Replacement text
    new_string: String,
    /// Replace all occurrences
    #[serde(default)]
    replace_all: Option<bool>,
}

pub struct EditTool;

#[async_trait]
impl BaseTool for EditTool {
    fn name(&self) -> &str { "edit" }
    fn description(&self) -> &str { "Replace an exact string in a file with new content." }
    fn params_schema(&self) -> Value { schema_for::<EditArgs>() }

    async fn run(&self, args: Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let args: EditArgs = serde_json::from_value(args)?;
        let content = fs::read_to_string(&args.file_path)?;

        if !content.contains(&args.old_string) {
            return Ok(format!("Error: '{}' not found in {}", args.old_string, args.file_path));
        }

        let new_content = if args.replace_all.unwrap_or(false) {
            content.replace(&args.old_string, &args.new_string)
        } else {
            content.replacen(&args.old_string, &args.new_string, 1)
        };

        fs::write(&args.file_path, &new_content)?;
        Ok(format!("Successfully edited {}", args.file_path))
    }
}

#[derive(Deserialize, JsonSchema)]
struct FindArgs {
    /// Glob pattern
    pattern: String,
    /// Root directory
    #[serde(default)]
    path: Option<String>,
}

pub struct FindTool;

#[async_trait]
impl BaseTool for FindTool {
    fn name(&self) -> &str { "find" }
    fn description(&self) -> &str { "Find files matching a glob pattern." }
    fn params_schema(&self) -> Value { schema_for::<FindArgs>() }

    async fn run(&self, args: Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let args: FindArgs = serde_json::from_value(args)?;
        let root = args.path.as_deref().unwrap_or(".");

        let full_pattern = if args.pattern.starts_with('/') || args.pattern.starts_with('.') {
            args.pattern
        } else {
            format!("{}/{}", root, args.pattern)
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

#[derive(Deserialize, JsonSchema)]
struct GrepArgs {
    /// Regex pattern to search for
    pattern: String,
    /// File or directory to search
    #[serde(default)]
    path: Option<String>,
    /// Glob for file filtering
    #[serde(default)]
    include: Option<String>,
}

pub struct GrepTool;

#[async_trait]
impl BaseTool for GrepTool {
    fn name(&self) -> &str { "grep" }
    fn description(&self) -> &str { "Search file contents for a regex pattern." }
    fn params_schema(&self) -> Value { schema_for::<GrepArgs>() }

    async fn run(&self, args: Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let args: GrepArgs = serde_json::from_value(args)?;
        let re = regex::Regex::new(&args.pattern)?;
        let mut results = Vec::new();

        let path = Path::new(args.path.as_deref().unwrap_or("."));
        if path.is_file() {
            grep_file(&re, path, &mut results)?;
        } else if path.is_dir() {
            grep_dir(&re, path, args.include.as_deref(), &mut results)?;
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

#[derive(Deserialize, JsonSchema)]
struct LsArgs {
    /// Directory to list
    #[serde(default)]
    path: Option<String>,
}

pub struct LsTool;

#[async_trait]
impl BaseTool for LsTool {
    fn name(&self) -> &str { "ls" }
    fn description(&self) -> &str { "List directory contents." }
    fn params_schema(&self) -> Value { schema_for::<LsArgs>() }

    async fn run(&self, args: Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let args: LsArgs = serde_json::from_value(args)?;
        let path = args.path.as_deref().unwrap_or(".");
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

#[derive(Deserialize, JsonSchema)]
struct ReadArgs {
    /// Path to the file
    file_path: String,
    /// Start line (1-indexed)
    #[serde(default)]
    offset: Option<usize>,
    /// Number of lines to read
    #[serde(default)]
    limit: Option<usize>,
}

pub struct ReadTool;

#[async_trait]
impl BaseTool for ReadTool {
    fn name(&self) -> &str { "read" }
    fn description(&self) -> &str { "Read a file and return its contents." }
    fn params_schema(&self) -> Value { schema_for::<ReadArgs>() }

    async fn run(&self, args: Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let args: ReadArgs = serde_json::from_value(args)?;
        let content = fs::read_to_string(&args.file_path)?;
        let lines: Vec<&str> = content.lines().collect();

        let start = args.offset.unwrap_or(1).saturating_sub(1);
        let end = match args.limit {
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

#[derive(Deserialize, JsonSchema)]
struct WriteArgs {
    /// Path to the file
    file_path: String,
    /// Content to write
    content: String,
}

pub struct WriteTool;

#[async_trait]
impl BaseTool for WriteTool {
    fn name(&self) -> &str { "write" }
    fn description(&self) -> &str { "Write content to a file, creating directories if needed." }
    fn params_schema(&self) -> Value { schema_for::<WriteArgs>() }

    async fn run(&self, args: Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let args: WriteArgs = serde_json::from_value(args)?;
        let path = Path::new(&args.file_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&args.file_path, &args.content)?;
        Ok(format!("Successfully wrote to {}", args.file_path))
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
